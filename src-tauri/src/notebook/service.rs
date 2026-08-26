// src-tauri/src/notebook/service.rs
use crate::error::{AppError, AppResult};
use crate::link::graph;
use crate::notebook::model::*;
use crate::storage::database::Database;
use crate::storage::markdown::MarkdownStorage;
use chrono::Utc;
use rusqlite::params;
use std::path::Path;
use uuid::Uuid;

pub struct NotebookService;

/// One word-count algorithm for every write path (create/update used to
/// disagree: whitespace tokens vs CJK-aware counting).
pub fn count_words(content: &str) -> i64 {
    let chinese = content
        .matches(|c: char| ('\u{4e00}'..='\u{9fa5}').contains(&c))
        .count();
    let english = content
        .split_whitespace()
        .filter(|w| {
            w.chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
        })
        .count();
    (chinese + english) as i64
}

/// Short plain-text preview used as the note summary.
pub fn summarize(content: &str) -> Option<String> {
    let preview: String = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("---"))
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect();
    if preview.is_empty() {
        None
    } else {
        Some(preview)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

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
        let word_count = count_words(&req.content);
        let summary = summarize(&req.content);

        let path = md.create_note(&req.folder, &req.title, &req.content)?;
        let relative_path = path
            .strip_prefix(md.base())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        {
            let conn = db.conn();
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO notes (id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0)",
                params![id, relative_path, req.title, req.folder, tags_json, now, now, word_count, summary],
            )?;
            tx.execute(
                "INSERT INTO notes_fts (rowid, title, content, tags) VALUES ((SELECT rowid FROM notes WHERE id=?1), ?2, ?3, ?4)",
                params![id, req.title, req.content, tags.join(", ")],
            )?;
            tx.commit()?;
        }
        graph::update_note_links(db, &id, &req.content)?;

        Ok(Note {
            id,
            path: relative_path,
            title: req.title,
            folder: req.folder,
            tags,
            created_at: now.clone(),
            updated_at: now,
            word_count,
            summary,
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
        if Self::is_deleted(db, &req.id)? {
            return Err(AppError::InvalidState("Note is in the trash".into()));
        }
        let now = Utc::now().to_rfc3339();

        // If title changed, rename the file. Compensate by renaming back if
        // the DB update fails, so disk and DB cannot drift apart silently.
        let mut renamed_from: Option<std::path::PathBuf> = None;
        if let Some(title) = &req.title {
            if *title != note.title {
                let new_path = md.move_note(&note.path, &note.folder, title)?;
                renamed_from = Some(md.full_path(&note.path));
                note.path = new_path;
                note.title = title.clone();
            }
        }

        if let Some(content) = &req.content {
            md.update_note(&note.path, content)?;
            note.word_count = count_words(content);
            note.summary = summarize(content);
        }
        if let Some(tags) = req.tags {
            note.tags = tags;
        }
        note.updated_at = now.clone();

        let new_content = req.content.clone();
        let result = Self::persist_note_update(db, &note, &now, new_content.as_deref());

        if let (Err(db_err), Some(old_abs)) = (&result, renamed_from) {
            // Roll the rename back so the old DB row still matches disk.
            let _ = std::fs::rename(md.full_path(&note.path), old_abs);
            return Err(db_err.clone());
        }
        result?;

        if let Some(content) = &new_content {
            graph::update_note_links(db, &note.id, content)?;
        }
        Ok(note)
    }

    fn persist_note_update(
        db: &Database,
        note: &Note,
        now: &str,
        content: Option<&str>,
    ) -> AppResult<()> {
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        let tags_json = serde_json::to_string(&note.tags)?;
        tx.execute(
            "UPDATE notes SET title=?1, path=?2, tags=?3, updated_at=?4, word_count=?5, summary=?6 WHERE id=?7",
            params![note.title, note.path, tags_json, now, note.word_count, note.summary, note.id],
        )?;
        if let Some(content) = content {
            tx.execute(
                "UPDATE notes_fts SET title=?1, content=?2, tags=?3 WHERE rowid=(SELECT rowid FROM notes WHERE id=?4)",
                params![note.title, content, note.tags.join(", "), note.id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Soft-delete: move the file into the trash directory and mark the row.
    /// The note disappears from every listing/search; `restore_note` undoes it.
    pub fn trash_note(
        db: &Database,
        md: &MarkdownStorage,
        id: &str,
        trash_dir: &Path,
    ) -> AppResult<()> {
        let note = Self::query_note_by_id(db, id)?;
        if Self::is_deleted(db, id)? {
            return Err(AppError::InvalidState(
                "Note is already in the trash".into(),
            ));
        }
        std::fs::create_dir_all(trash_dir)?;
        std::fs::rename(md.full_path(&note.path), trash_dir.join(format!("{id}.md")))?;

        let now = Utc::now().to_rfc3339();
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE notes SET deleted_at=?1 WHERE id=?2",
            params![now, id],
        )?;
        tx.execute(
            "DELETE FROM notes_fts WHERE rowid=(SELECT rowid FROM notes WHERE id=?1)",
            params![id],
        )?;
        tx.commit().inspect_err(|_e| {
            // Compensate: put the file back so the row stays consistent.
            let _ = std::fs::rename(trash_dir.join(format!("{id}.md")), md.full_path(&note.path));
        })?;
        Ok(())
    }

    /// Restore a trashed note to its original location.
    pub fn restore_note(
        db: &Database,
        md: &MarkdownStorage,
        id: &str,
        trash_dir: &Path,
    ) -> AppResult<Note> {
        let note = Self::query_note_by_id(db, id)?;
        if !Self::is_deleted(db, id)? {
            return Err(AppError::InvalidState("Note is not in the trash".into()));
        }
        let trash_file = trash_dir.join(format!("{id}.md"));
        if !trash_file.exists() {
            return Err(AppError::NotFound(format!("Trashed file missing: {id}")));
        }
        let dest = md.full_path(&note.path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&trash_file, &dest)?;

        let content = md.read_note(&note.path).unwrap_or_default();
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute("UPDATE notes SET deleted_at=NULL WHERE id=?1", params![id])?;
        let tags_json = serde_json::to_string(&note.tags)?;
        tx.execute(
            "INSERT INTO notes_fts (rowid, title, content, tags)
             SELECT rowid, ?1, ?2, ?3 FROM notes WHERE id=?4",
            params![note.title, content, tags_json, id],
        )?;
        tx.commit().inspect_err(|_e| {
            let _ = std::fs::rename(&dest, &trash_file);
        })?;
        graph::update_note_links(db, id, &content)?;
        Ok(note)
    }

    /// Permanently delete a trashed note (file + row; links cascade).
    pub fn purge_note(db: &Database, id: &str, trash_dir: &Path) -> AppResult<()> {
        if !Self::is_deleted(db, id)? {
            return Err(AppError::InvalidState("Note is not in the trash".into()));
        }
        let _ = std::fs::remove_file(trash_dir.join(format!("{id}.md")));
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM notes_fts WHERE rowid=(SELECT rowid FROM notes WHERE id=?1)",
            params![id],
        )?;
        tx.execute("DELETE FROM notes WHERE id=?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// All trashed notes, newest deletion first.
    pub fn list_trash(db: &Database) -> AppResult<Vec<TrashItem>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, title, folder, deleted_at FROM notes
             WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
        )?;
        let items = stmt
            .query_map([], |row| {
                Ok(TrashItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    folder: row.get(2)?,
                    deleted_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// Empty the trash permanently. Returns the number of purged notes.
    pub fn empty_trash(db: &Database, trash_dir: &Path) -> AppResult<usize> {
        let items = Self::list_trash(db)?;
        let mut purged = 0;
        for item in items {
            if Self::purge_note(db, &item.id, trash_dir).is_ok() {
                purged += 1;
            }
        }
        Ok(purged)
    }

    /// Backlinks, outgoing links, and unresolved [[targets]] for one note.
    /// Backlinks/outgoing only include live (non-trashed) notes.
    pub fn get_note_links(db: &Database, md: &MarkdownStorage, id: &str) -> AppResult<NoteLinks> {
        let note = Self::query_note_by_id(db, id)?;
        let conn = db.conn();

        let mut backlinks = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.folder, l.context FROM links l                  JOIN notes n ON n.id = l.source_note_id                  WHERE l.target_note_id = ?1 AND n.deleted_at IS NULL                  ORDER BY n.updated_at DESC",
            )?;
            let rows = stmt.query_map(params![id], |row| {
                Ok(NoteLinkItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    folder: row.get(2)?,
                    context: row.get(3).unwrap_or_default(),
                })
            })?;
            for r in rows {
                backlinks.push(r?);
            }
        }

        let mut outgoing = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.folder, l.context FROM links l                  JOIN notes n ON n.id = l.target_note_id                  WHERE l.source_note_id = ?1 AND n.deleted_at IS NULL                  ORDER BY n.title ASC",
            )?;
            let rows = stmt.query_map(params![id], |row| {
                Ok(NoteLinkItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    folder: row.get(2)?,
                    context: row.get(3).unwrap_or_default(),
                })
            })?;
            for r in rows {
                outgoing.push(r?);
            }
        }

        // Unresolved: [[targets]] in this note that no live note has as title.
        let mut unresolved = Vec::new();
        {
            let content = md.read_note(&note.path).unwrap_or_default();
            let mut titles = std::collections::HashSet::new();
            let mut stmt = conn.prepare("SELECT title FROM notes WHERE deleted_at IS NULL")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            for t in rows.flatten() {
                titles.insert(t);
            }
            let mut seen = std::collections::HashSet::new();
            for target in crate::link::parser::LinkParser::extract_links(&content) {
                if !titles.contains(&target) && seen.insert(target.clone()) {
                    unresolved.push(target);
                }
            }
        }

        Ok(NoteLinks {
            backlinks,
            outgoing,
            unresolved,
        })
    }

    pub fn is_deleted(db: &Database, id: &str) -> AppResult<bool> {
        let deleted: Option<String> = db.conn().query_row(
            "SELECT deleted_at FROM notes WHERE id=?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(deleted.is_some())
    }

    pub fn move_note(db: &Database, md: &MarkdownStorage, req: MoveNoteRequest) -> AppResult<Note> {
        let note = Self::query_note_by_id(db, &req.id)?;
        if Self::is_deleted(db, &req.id)? {
            return Err(AppError::InvalidState("Note is in the trash".into()));
        }
        let new_title = req.new_title.unwrap_or_else(|| note.title.clone());
        let old_path = note.path.clone();
        let new_path = md.move_note(&note.path, &req.target_folder, &new_title)?;

        let now = Utc::now().to_rfc3339();
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE notes SET path=?1, title=?2, folder=?3, updated_at=?4 WHERE id=?5",
            params![new_path, new_title, req.target_folder, now, note.id],
        )?;
        if new_title != note.title {
            tx.execute(
                "UPDATE notes_fts SET title=?1 WHERE rowid=(SELECT rowid FROM notes WHERE id=?2)",
                params![new_title, note.id],
            )?;
        }
        tx.commit().inspect_err(|_e| {
            // Compensate: put the file back so the DB row stays valid.
            let _ = std::fs::rename(md.full_path(&new_path), md.full_path(&old_path));
        })?;

        Ok(Note {
            path: new_path,
            title: new_title,
            folder: req.target_folder,
            updated_at: now,
            ..note
        })
    }

    /// Rename a folder on disk and update every affected DB row (folder paths
    /// and note paths, prefix-replaced) in one transaction.
    pub fn rename_folder(
        db: &Database,
        md: &MarkdownStorage,
        path: &str,
        new_name: &str,
    ) -> AppResult<String> {
        if path.is_empty() {
            return Err(AppError::InvalidPath("Cannot rename the vault root".into()));
        }
        let new_path = md.rename_folder(path, new_name)?;
        let old_len = path.chars().count() as i64;
        let old_child_prefix = format!("{}/", path);
        let new_child_prefix = format!("{}/", new_path);

        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        // Notes directly in the folder.
        tx.execute(
            "UPDATE notes SET folder=?1 WHERE folder=?2",
            params![new_path, path],
        )?;
        // Notes in subfolders: replace the leading "old/" with "new/".
        tx.execute(
            "UPDATE notes SET folder=?1 || substr(folder, ?2) WHERE substr(folder, 1, ?3) = ?4",
            params![new_child_prefix, old_len + 2, old_len + 1, old_child_prefix],
        )?;
        tx.execute(
            "UPDATE notes SET path=?1 || substr(path, ?2) WHERE substr(path, 1, ?3) = ?4",
            params![new_child_prefix, old_len + 2, old_len + 1, old_child_prefix],
        )?;
        tx.commit().inspect_err(|_e| {
            let _ = md.rename_folder(&new_path, path.split('/').next_back().unwrap_or(path));
        })?;
        Ok(new_path)
    }

    /// Soft-delete a folder: every note inside goes to the trash (restorable
    /// with its original folder path), then the emptied directory is removed.
    pub fn delete_folder(
        db: &Database,
        md: &MarkdownStorage,
        path: &str,
        trash_dir: &Path,
    ) -> AppResult<()> {
        if path.is_empty() {
            return Err(AppError::InvalidPath("Cannot delete the vault root".into()));
        }
        let old_len = path.chars().count() as i64;
        let child_prefix = format!("{}/", path);

        let conn = db.conn();
        let ids: Vec<String> = {
            let mut stmt = conn.prepare(
                "SELECT id FROM notes WHERE deleted_at IS NULL AND (folder = ?1 OR substr(folder, 1, ?2) = ?3)",
            )?;
            let rows = stmt.query_map(params![path, old_len + 1, child_prefix], |r| r.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        if ids.is_empty() {
            md.delete_folder(path)?;
            return Ok(());
        }

        // Move files to the trash first, then mark rows in one transaction.
        std::fs::create_dir_all(trash_dir)?;
        let now = Utc::now().to_rfc3339();
        let mut moved: Vec<(String, String)> = Vec::new(); // (id, original rel path)
        for id in &ids {
            let note = Self::query_note_by_id(db, id)?;
            let dest = trash_dir.join(format!("{id}.md"));
            std::fs::rename(md.full_path(&note.path), &dest)?;
            moved.push((id.clone(), note.path));
        }

        let tx = conn.unchecked_transaction()?;
        let mut db_ok = true;
        for id in &ids {
            if tx
                .execute(
                    "UPDATE notes SET deleted_at=?1 WHERE id=?2",
                    params![now, id],
                )
                .is_err()
            {
                db_ok = false;
                break;
            }
            let _ = tx.execute(
                "DELETE FROM notes_fts WHERE rowid=(SELECT rowid FROM notes WHERE id=?1)",
                params![id],
            );
        }
        if db_ok {
            tx.commit()?;
        } else {
            let _ = tx.rollback();
            // Compensate: put every file back.
            for (id, rel) in &moved {
                let _ = std::fs::rename(trash_dir.join(format!("{id}.md")), md.full_path(rel));
            }
            return Err(AppError::Database("Failed to trash folder notes".into()));
        }

        // Directory is empty of trashed notes; remove it (attachments and
        // unrelated files inside are gone with it, same as before).
        let _ = md.delete_folder(path);
        Ok(())
    }

    pub fn list_notes(db: &Database, md: &MarkdownStorage, folder: &str) -> AppResult<Vec<Note>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE folder = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC"
        )?;
        let mut notes = stmt
            .query_map(params![folder], row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;

        // Backfill missing summaries once and persist them, so we never
        // re-read the whole folder's files on every listing again.
        for note in &mut notes {
            if note.summary.is_none() {
                if let Ok(content) = md.read_note(&note.path) {
                    if let Some(preview) = summarize(&content) {
                        let _ = db.conn().execute(
                            "UPDATE notes SET summary=?1 WHERE id=?2 AND summary IS NULL",
                            params![preview, note.id],
                        );
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
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }

        let conn = db.conn();
        let like_pattern = format!(
            "%{}%",
            query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let mut results: Vec<SearchResult> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // 1) Title and tags are short indexed-ish columns; LIKE is fine here.
        match scope {
            "content" => {}
            _ => {
                let title_clause = "title LIKE ?1 ESCAPE '\\'";
                let tags_clause = "tags LIKE ?1 ESCAPE '\\'";
                let where_sql = match scope {
                    "title" => title_clause.to_string(),
                    "tags" => tags_clause.to_string(),
                    _ => format!("{} OR {}", title_clause, tags_clause),
                };
                let sql = format!(
                    "SELECT id, title, path FROM notes WHERE deleted_at IS NULL AND {} ORDER BY updated_at DESC LIMIT 50",
                    where_sql
                );
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map(params![like_pattern], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;
                for r in rows {
                    let (id, title, path) = r?;
                    if seen.insert(id.clone()) {
                        results.push(SearchResult {
                            id,
                            title,
                            path,
                            snippet: String::new(),
                            rank: 99.0,
                        });
                    }
                }
            }
        }

        // 2) Content search. With the trigram tokenizer, MATCH does indexed
        //    substring search (including CJK) and gives us a highlighted
        //    snippet. Trigram needs >= 3 code points, so shorter queries fall
        //    back to a LIKE scan.
        match scope {
            "title" | "tags" => {}
            _ => {
                if query.chars().count() >= 3 {
                    let fts_query = format!("\"{}\"", query.replace('"', "\"\""));
                    let mut stmt = conn.prepare(
                        "SELECT notes.id, notes.title, notes.path, \
                                snippet(notes_fts, 1, char(1), char(2), '…', 16) \
                         FROM notes_fts JOIN notes ON notes.rowid = notes_fts.rowid \
                         WHERE notes.deleted_at IS NULL AND notes_fts MATCH ?1 ORDER BY rank LIMIT 50",
                    )?;
                    let rows = stmt.query_map(params![fts_query], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                        ))
                    })?;
                    for r in rows {
                        let (id, title, path, raw_snippet) = r?;
                        if seen.insert(id.clone()) {
                            // Escape note text, then turn the control-char
                            // markers into <mark> tags — safe v-html payload.
                            let snippet = html_escape(&raw_snippet)
                                .replace('\u{1}', "<mark>")
                                .replace('\u{2}', "</mark>");
                            results.push(SearchResult {
                                id,
                                title,
                                path,
                                snippet,
                                rank: 50.0,
                            });
                        }
                    }
                } else {
                    let mut stmt = conn.prepare(
                        "SELECT notes.id, notes.title, notes.path FROM notes \
                         WHERE notes.id IN (
                             SELECT n2.id FROM notes n2 JOIN notes_fts f ON n2.rowid = f.rowid \
                             WHERE f.content LIKE ?1 ESCAPE '\\' AND n2.deleted_at IS NULL) \
                         ORDER BY notes.updated_at DESC LIMIT 50",
                    )?;
                    let rows = stmt.query_map(params![like_pattern], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })?;
                    for r in rows {
                        let (id, title, path) = r?;
                        if seen.insert(id.clone()) {
                            results.push(SearchResult {
                                id,
                                title,
                                path,
                                snippet: String::new(),
                                rank: 50.0,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn list_recent_notes(db: &Database, limit: i64) -> AppResult<Vec<Note>> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE deleted_at IS NULL ORDER BY updated_at DESC LIMIT ?1"
        )?;
        let notes = stmt
            .query_map(params![limit], row_to_note)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(notes)
    }

    /// Full FTS rebuild: recreate notes_fts with the trigram tokenizer and
    /// reload every note's content from disk. Used when migrating an old
    /// default-tokenizer table.
    pub fn rebuild_fts(db: &Database, md: &MarkdownStorage) -> AppResult<()> {
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute("DROP TABLE IF EXISTS notes_fts", [])?;
        tx.execute(
            "CREATE VIRTUAL TABLE notes_fts USING fts5(title, content, tags, tokenize='trigram')",
            [],
        )?;
        let mut stmt = tx.prepare("SELECT rowid, path, title, tags FROM notes")?;
        let notes: Vec<(i64, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        for (rowid, path, title, tags_json) in notes {
            let content = md.read_note(&path).unwrap_or_default();
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            tx.execute(
                "INSERT INTO notes_fts (rowid, title, content, tags) VALUES (?1, ?2, ?3, ?4)",
                params![rowid, title, content, tags.join(", ")],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn query_note_by_id(db: &Database, id: &str) -> AppResult<Note> {
        let mut stmt = db.conn().prepare(
            "SELECT id, path, title, folder, tags, created_at, updated_at, word_count, summary, ai_categorized
             FROM notes WHERE id = ?1"
        )?;
        stmt.query_row(params![id], row_to_note)
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AppError::NotFound(format!("Note not found: {}", id))
                }
                other => AppError::from(other),
            })
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

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        path: row.get("path")?,
        title: row.get("title")?,
        folder: row.get("folder")?,
        tags: serde_json::from_str(&row.get::<_, String>("tags")?).unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        word_count: row.get::<_, Option<i64>>("word_count")?.unwrap_or(0),
        summary: row.get("summary")?,
        ai_categorized: row
            .get::<_, Option<bool>>("ai_categorized")?
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_env() -> (Database, MarkdownStorage, PathBufHelper) {
        let dir = std::env::temp_dir().join(format!("taxa-svc-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Database::new(&dir.join("test.db")).unwrap();
        let md = MarkdownStorage::new(dir.join("notes"));
        std::fs::create_dir_all(dir.join("notes")).unwrap();
        (db, md, PathBufHelper(dir))
    }

    struct PathBufHelper(std::path::PathBuf);
    impl Drop for PathBufHelper {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn make_note(
        db: &Database,
        md: &MarkdownStorage,
        title: &str,
        folder: &str,
        content: &str,
    ) -> Note {
        NotebookService::create_note(
            db,
            md,
            CreateNoteRequest {
                folder: folder.into(),
                title: title.into(),
                content: content.into(),
                tags: Some(vec!["测试".into()]),
            },
        )
        .unwrap()
    }

    #[test]
    fn search_uses_fts_with_snippet() {
        let (db, md, _tmp) = test_env();
        make_note(
            &db,
            &md,
            "游泳技巧",
            "",
            "# 游泳\n\n自由泳换气的时候要转头。蛙泳腿是主要动力。",
        );
        make_note(&db, &md, "工作日志", "", "今天写了很多代码。");

        let results = NotebookService::search_notes(&db, "自由泳换气", "all").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "游泳技巧");
        assert!(
            results[0].snippet.contains("<mark>"),
            "snippet: {}",
            results[0].snippet
        );

        // Short CJK queries fall back to LIKE and still match.
        let short = NotebookService::search_notes(&db, "游泳", "all").unwrap();
        assert!(short.iter().any(|r| r.title == "游泳技巧"));

        // Tag search still works.
        let by_tag = NotebookService::search_notes(&db, "测试", "tags").unwrap();
        assert_eq!(by_tag.len(), 2);
    }

    #[test]
    fn rename_folder_updates_db_rows() {
        let (db, md, _tmp) = test_env();
        make_note(&db, &md, "A", "docs", "content a");
        make_note(&db, &md, "B", "docs/inner", "content b");
        make_note(&db, &md, "C", "other", "content c");

        NotebookService::rename_folder(&db, &md, "docs", "documents").unwrap();

        let docs = NotebookService::list_notes(&db, &md, "documents").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "A");
        assert!(docs[0].path.starts_with("documents/"));

        let inner = NotebookService::list_notes(&db, &md, "documents/inner").unwrap();
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0].title, "B");
        assert!(inner[0].path.starts_with("documents/inner/"));

        // Unrelated folder untouched.
        let other = NotebookService::list_notes(&db, &md, "other").unwrap();
        assert_eq!(other.len(), 1);

        // Old folder is gone from disk.
        assert!(!md.base().join("docs").exists());
    }

    #[test]
    fn delete_folder_soft_deletes() {
        let (db, md, tmp) = test_env();
        let trash = tmp.0.join("trash");
        make_note(&db, &md, "A", "gone", "content a");
        make_note(&db, &md, "B", "gone/deep", "content b");
        make_note(&db, &md, "C", "stay", "content c");

        NotebookService::delete_folder(&db, &md, "gone", &trash).unwrap();

        assert!(NotebookService::list_notes(&db, &md, "gone")
            .unwrap()
            .is_empty());
        assert!(NotebookService::list_notes(&db, &md, "gone/deep")
            .unwrap()
            .is_empty());
        assert_eq!(
            NotebookService::list_notes(&db, &md, "stay").unwrap().len(),
            1
        );
        assert!(!md.base().join("gone").exists());
        // Both notes are restorable, not gone.
        assert_eq!(NotebookService::list_trash(&db).unwrap().len(), 2);
    }

    #[test]
    fn update_note_with_empty_content_option_keeps_file() {
        let (db, md, _tmp) = test_env();
        let note = make_note(&db, &md, "T", "", "original body");
        // Title-only update (no content) must not touch file contents.
        let updated = NotebookService::update_note(
            &db,
            &md,
            UpdateNoteRequest {
                id: note.id.clone(),
                title: Some("T2".into()),
                content: None,
                folder: None,
                tags: None,
            },
        )
        .unwrap();
        assert_eq!(updated.title, "T2");
        let content = md.read_note(&updated.path).unwrap();
        assert!(content.contains("original body"));
    }

    #[test]
    fn word_count_is_cjk_aware() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("这是中文内容"), 6);
        // 4 CJK chars (混合内容) + 1 English word (text)
        assert_eq!(count_words("混合 text 内容"), 5);
    }

    #[test]
    fn trash_restore_roundtrip() {
        let (db, md, dir) = test_env();
        let trash = dir.0.join("trash");
        let note = make_note(&db, &md, "T", "", "unique findable content");
        make_note(&db, &md, "Other", "", "unrelated");

        // Soft delete: disappears from listings and search.
        NotebookService::trash_note(&db, &md, &note.id, &trash).unwrap();
        let remaining = NotebookService::list_notes(&db, &md, "").unwrap();
        assert!(!remaining.iter().any(|n| n.id == note.id));
        assert!(remaining.iter().any(|n| n.title == "Other"));
        assert!(
            NotebookService::search_notes(&db, "unique findable", "content")
                .unwrap()
                .is_empty()
        );
        assert_eq!(NotebookService::list_trash(&db).unwrap().len(), 1);
        assert!(!md.base().join("T.md").exists());
        assert!(trash.join(format!("{}.md", note.id)).exists());

        // Update on a trashed note is rejected.
        let err = NotebookService::update_note(
            &db,
            &md,
            UpdateNoteRequest {
                id: note.id.clone(),
                title: None,
                content: Some("x".into()),
                folder: None,
                tags: None,
            },
        );
        assert!(err.is_err());

        // Restore: file back, searchable again, links recomputed.
        let restored = NotebookService::restore_note(&db, &md, &note.id, &trash).unwrap();
        assert_eq!(restored.title, "T");
        assert!(md.read_note("T.md").unwrap().contains("unique findable"));
        assert_eq!(
            NotebookService::search_notes(&db, "unique findable", "content")
                .unwrap()
                .len(),
            1
        );
        assert!(NotebookService::list_trash(&db).unwrap().is_empty());
    }

    #[test]
    fn trash_purge_and_empty() {
        let (db, md, dir) = test_env();
        let trash = dir.0.join("trash");
        let a = make_note(&db, &md, "A", "", "a");
        let b = make_note(&db, &md, "B", "", "b");
        NotebookService::trash_note(&db, &md, &a.id, &trash).unwrap();
        NotebookService::trash_note(&db, &md, &b.id, &trash).unwrap();

        NotebookService::purge_note(&db, &a.id, &trash).unwrap();
        assert_eq!(NotebookService::list_trash(&db).unwrap().len(), 1);
        assert!(NotebookService::purge_note(&db, &a.id, &trash).is_err()); // already purged

        let purged = NotebookService::empty_trash(&db, &trash).unwrap();
        assert_eq!(purged, 1);
        assert!(NotebookService::list_trash(&db).unwrap().is_empty());
    }

    #[test]
    fn trash_folder_soft_deletes_subtree() {
        let (db, md, dir) = test_env();
        let trash = dir.0.join("trash");
        make_note(&db, &md, "A", "gone", "a");
        make_note(&db, &md, "B", "gone/deep", "b");
        make_note(&db, &md, "C", "stay", "c");

        NotebookService::delete_folder(&db, &md, "gone", &trash).unwrap();
        assert!(!md.base().join("gone").exists());
        assert_eq!(NotebookService::list_trash(&db).unwrap().len(), 2);
        // The staying note is untouched.
        assert_eq!(
            NotebookService::list_notes(&db, &md, "stay").unwrap().len(),
            1
        );

        // Restoring a subtree note brings the file back to its nested path.
        let trashed: Vec<_> = NotebookService::list_trash(&db).unwrap();
        let deep = trashed.iter().find(|t| t.title == "B").unwrap();
        let restored = NotebookService::restore_note(&db, &md, &deep.id, &trash).unwrap();
        assert_eq!(restored.folder, "gone/deep");
        assert!(md.base().join("gone").join("deep").join("B.md").exists());
    }

    #[test]
    fn note_links_panel_data() {
        let (db, md, _dir) = test_env();
        let a = make_note(&db, &md, "A", "", "see [[B]] and [[Missing]]");
        let b = make_note(&db, &md, "B", "", "back to [[A]]");
        // A was created before B existed; touch A so its [[B]] link resolves.
        NotebookService::update_note(
            &db,
            &md,
            UpdateNoteRequest {
                id: a.id.clone(),
                title: None,
                content: Some("see [[B]] and [[Missing]]".into()),
                folder: None,
                tags: None,
            },
        )
        .unwrap();

        let links = NotebookService::get_note_links(&db, &md, &a.id).unwrap();
        assert_eq!(links.backlinks.len(), 1);
        assert_eq!(links.backlinks[0].title, "B");
        assert_eq!(links.outgoing.len(), 1);
        assert_eq!(links.outgoing[0].title, "B");
        assert_eq!(links.unresolved, vec!["Missing".to_string()]);

        let links_b = NotebookService::get_note_links(&db, &md, &b.id).unwrap();
        assert_eq!(links_b.backlinks[0].title, "A");
        assert!(links_b.unresolved.is_empty());
    }

    #[test]
    fn fts_rebuild_works() {
        let (db, md, _tmp) = test_env();
        make_note(&db, &md, "A", "", "独一无二的内容XYZ");
        // Simulate a legacy table by forcing a rebuild.
        NotebookService::rebuild_fts(&db, &md).unwrap();
        let results = NotebookService::search_notes(&db, "独一无二的内容", "content").unwrap();
        assert_eq!(results.len(), 1);
    }
}
