// src-tauri/src/notebook/service.rs
use crate::error::{AppError, AppResult};
use crate::notebook::model::*;
use crate::storage::database::Database;
use crate::storage::markdown::MarkdownStorage;
use chrono::Utc;
use rusqlite::params;
use uuid::Uuid;

pub struct NotebookService;

impl NotebookService {
    pub fn create_note(
        db: &Database,
        md: &MarkdownStorage,
        req: CreateNoteRequest,
    ) -> AppResult<Note> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tags = req.tags.unwrap_or_default();
        let tags_json = serde_json::to_string(&tags)?;
        let word_count = req.content.split_whitespace().count() as i64;

        let path = md.create_note(&req.folder, &req.title, &req.content)?;
        let relative_path = path
            .strip_prefix(md.full_path(""))
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        db.conn().execute(
            "INSERT INTO notes (id, path, title, folder, tags, created_at, updated_at, word_count, ai_categorized)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
            params![id, relative_path, req.title, req.folder, tags_json, now, now, word_count],
        )?;

        let fts_content = req.content.clone();
        db.conn().execute(
            "INSERT INTO notes_fts (rowid, title, content, tags) VALUES ((SELECT rowid FROM notes WHERE id=?1), ?2, ?3, ?4)",
            params![id, req.title, fts_content, tags.join(", ")],
        )?;

        Ok(Note {
            id,
            path: relative_path,
            title: req.title,
            folder: req.folder,
            tags,
            created_at: now.clone(),
            updated_at: now,
            word_count,
            summary: None,
            ai_categorized: false,
        })
    }

    pub fn get_note(db: &Database, md: &MarkdownStorage, id: &str) -> AppResult<(Note, String)> {
        let note = Self::query_note_by_id(db, id)?;
        let content = md.read_note(&note.path)?;
        Ok((note, content))
    }

    pub fn update_note(
        db: &Database,
        md: &MarkdownStorage,
        req: UpdateNoteRequest,
    ) -> AppResult<Note> {
        let mut note = Self::query_note_by_id(db, &req.id)?;
        let now = Utc::now().to_rfc3339();

        // If title changed, rename the file
        if let Some(title) = &req.title {
            if *title != note.title {
                let new_path = md.move_note(&note.path, &note.folder, title)?;
                note.path = new_path;
                note.title = title.clone();
            }
        }

        if let Some(content) = &req.content {
            md.update_note(&note.path, content)?;
            // Count Chinese characters + English words
            let chinese = content.matches(|c: char| c >= '\u{4e00}' && c <= '\u{9fa5}').count() as i64;
            let english = content.split_whitespace()
                .filter(|w| w.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false))
                .count() as i64;
            note.word_count = chinese + english;
        }
        if let Some(tags) = req.tags {
            note.tags = tags;
        }
        note.updated_at = now.clone();

        let tags_json = serde_json::to_string(&note.tags)?;
        db.conn().execute(
            "UPDATE notes SET title=?1, path=?2, tags=?3, updated_at=?4, word_count=?5 WHERE id=?6",
            params![note.title, note.path, tags_json, now, note.word_count, note.id],
        )?;

        if let Some(content) = req.content {
            db.conn().execute(
                "UPDATE notes_fts SET title=?1, content=?2, tags=?3 WHERE rowid=(SELECT rowid FROM notes WHERE id=?4)",
                params![note.title, content, note.tags.join(", "), note.id],
            )?;
        }

        Ok(note)
    }

    pub fn delete_note(db: &Database, md: &MarkdownStorage, id: &str) -> AppResult<()> {
        let note = Self::query_note_by_id(db, id)?;
        md.delete_note(&note.path)?;
        db.conn().execute("DELETE FROM notes_fts WHERE rowid=(SELECT rowid FROM notes WHERE id=?1)", params![id])?;
        db.conn().execute("DELETE FROM notes WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn move_note(
        db: &Database,
        md: &MarkdownStorage,
        req: MoveNoteRequest,
    ) -> AppResult<Note> {
        let note = Self::query_note_by_id(db, &req.id)?;
        let new_title = req.new_title.unwrap_or_else(|| note.title.clone());
        let new_path = md.move_note(&note.path, &req.target_folder, &new_title)?;

        let now = Utc::now().to_rfc3339();
        db.conn().execute(
            "UPDATE notes SET path=?1, title=?2, folder=?3, updated_at=?4 WHERE id=?5",
            params![new_path, new_title, req.target_folder, now, note.id],
        )?;

        Ok(Note {
            path: new_path,
            title: new_title,
            folder: req.target_folder,
            updated_at: now,
            ..note
        })
    }

    pub fn list_notes(db: &Database, md: &MarkdownStorage, folder: &str) -> AppResult<Vec<Note>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE folder = ?1 ORDER BY updated_at DESC"
        )?;
        let mut notes = stmt.query_map(params![folder], |row| {
            Ok(row_to_note(row))
        })?.collect::<Result<Vec<_>, _>>()?;

        // Auto-fill summary from file content when missing
        for note in &mut notes {
            if note.summary.is_none() {
                if let Ok(content) = md.read_note(&note.path) {
                    let preview: String = content
                        .lines()
                        .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("---"))
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .chars()
                        .take(120)
                        .collect();
                    if !preview.is_empty() {
                        note.summary = Some(preview);
                    }
                }
            }
        }

        Ok(notes)
    }

    pub fn get_folder_tree(md: &MarkdownStorage) -> AppResult<Vec<Folder>> {
        Self::build_folder_tree(md, "")
    }

    pub fn search_notes(db: &Database, query: &str, scope: &str) -> AppResult<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let conn = db.conn();
        let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut results: Vec<SearchResult> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // 1) Search notes table for title and tags (no FTS needed)
        match scope {
            "content" => {} // skip title/tags for content-only
            _ => {
                let title_clause = "title LIKE ?1 ESCAPE '\\'";
                let tags_clause = "tags LIKE ?1 ESCAPE '\\'";
                let where_sql = match scope {
                    "title" => title_clause.to_string(),
                    "tags" => tags_clause.to_string(),
                    _ => format!("{} OR {}", title_clause, tags_clause),
                };
                let sql = format!(
                    "SELECT id, title, path FROM notes WHERE {} ORDER BY updated_at DESC LIMIT 50",
                    where_sql
                );
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map(params![like_pattern], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
                })?;
                for r in rows {
                    let (id, title, path) = r?;
                    if seen.insert(id.clone()) {
                        results.push(SearchResult { id, title, path, snippet: String::new(), rank: 99.0 });
                    }
                }
            }
        }

        // 2) Search FTS for content (LIKE on f.content)
        match scope {
            "title" | "tags" => {} // skip content for title/tags-only
            _ => {
                let sql = "SELECT n.id, n.title, n.path FROM notes n WHERE n.id IN \
                           (SELECT n2.id FROM notes n2 JOIN notes_fts f ON n2.rowid = f.rowid \
                            WHERE f.content LIKE ?1 ESCAPE '\\') \
                           ORDER BY n.updated_at DESC LIMIT 50";
                let mut stmt = conn.prepare(sql)?;
                let rows = stmt.query_map(params![like_pattern], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
                })?;
                for r in rows {
                    let (id, title, path) = r?;
                    if seen.insert(id.clone()) {
                        results.push(SearchResult { id, title, path, snippet: String::new(), rank: 50.0 });
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn list_recent_notes(db: &Database, limit: i64) -> AppResult<Vec<Note>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes ORDER BY updated_at DESC LIMIT ?1"
        )?;
        let notes = stmt.query_map(params![limit], |row| {
            Ok(row_to_note(row))
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    fn query_note_by_id(db: &Database, id: &str) -> AppResult<Note> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE id = ?1"
        )?;
        let note = stmt.query_row(params![id], |row| Ok(row_to_note(row)))
            .map_err(|_| AppError::NotFound(format!("Note not found: {}", id)))?;
        Ok(note)
    }

    fn build_folder_tree(md: &MarkdownStorage, prefix: &str) -> AppResult<Vec<Folder>> {
        let subfolders = md.list_subfolders(prefix)?;
        let mut result = Vec::new();
        for name in subfolders {
            let folder_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };
            let children = Self::build_folder_tree(md, &folder_path)?;
            let note_files = md.list_folder(&folder_path)?;
            let note_count = note_files.len() as i64;
            result.push(Folder {
                name,
                path: folder_path,
                children,
                note_count,
            });
        }
        Ok(result)
    }
}

fn row_to_note(row: &rusqlite::Row) -> Note {
    Note {
        id: row.get("id").unwrap(),
        path: row.get("path").unwrap(),
        title: row.get("title").unwrap(),
        folder: row.get("folder").unwrap(),
        tags: serde_json::from_str(&row.get::<_, String>("tags").unwrap_or_default()).unwrap_or_default(),
        created_at: row.get("created_at").unwrap(),
        updated_at: row.get("updated_at").unwrap(),
        word_count: row.get("word_count").unwrap_or(0),
        summary: row.get("summary").unwrap_or(None),
        ai_categorized: row.get("ai_categorized").unwrap_or(false),
    }
}
