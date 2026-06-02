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

        if let Some(title) = req.title {
            note.title = title;
        }
        if let Some(content) = &req.content {
            md.update_note(&note.path, content)?;
            note.word_count = content.split_whitespace().count() as i64;
        }
        if let Some(tags) = req.tags {
            note.tags = tags;
        }
        note.updated_at = now.clone();

        let tags_json = serde_json::to_string(&note.tags)?;
        db.conn().execute(
            "UPDATE notes SET title=?1, tags=?2, updated_at=?3, word_count=?4 WHERE id=?5",
            params![note.title, tags_json, now, note.word_count, note.id],
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

    pub fn list_notes(db: &Database, folder: &str) -> AppResult<Vec<Note>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE folder = ?1 ORDER BY updated_at DESC"
        )?;
        let notes = stmt.query_map(params![folder], |row| {
            Ok(row_to_note(row))
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    pub fn get_folder_tree(md: &MarkdownStorage) -> AppResult<Vec<Folder>> {
        Self::build_folder_tree(md, "")
    }

    pub fn search_notes(db: &Database, query: &str) -> AppResult<Vec<SearchResult>> {
        let mut stmt = db.conn().prepare(
            "SELECT n.id, n.title, n.path, snippet(notes_fts, 1, '<mark>', '</mark>', '...', 32) as snippet, rank
             FROM notes_fts f JOIN notes n ON n.rowid = f.rowid
             WHERE notes_fts MATCH ?1 ORDER BY rank LIMIT 50"
        )?;
        let results = stmt.query_map(params![query], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                title: row.get(1)?,
                path: row.get(2)?,
                snippet: row.get(3)?,
                rank: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(results)
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
