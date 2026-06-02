// src-tauri/src/storage/database.rs
use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .map_err(|e| AppError::Database(format!("Failed to open database: {}", e)))?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> AppResult<()> {
        self.conn
            .execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| AppError::Database(e.to_string()))?;

        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                folder TEXT NOT NULL,
                tags TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                word_count INTEGER DEFAULT 0,
                summary TEXT,
                ai_categorized INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS links (
                source_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                target_note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                context TEXT,
                PRIMARY KEY (source_note_id, target_note_id)
            );

            CREATE TABLE IF NOT EXISTS ai_operations (
                id TEXT PRIMARY KEY,
                note_id TEXT REFERENCES notes(id) ON DELETE SET NULL,
                operation_type TEXT NOT NULL,
                before_state TEXT,
                after_state TEXT,
                status TEXT DEFAULT 'pending',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS llm_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                api_url TEXT NOT NULL,
                api_key_encrypted TEXT,
                model_name TEXT NOT NULL,
                is_default INTEGER DEFAULT 0,
                enabled INTEGER DEFAULT 1
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(title, content, tags);

            CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder);
            CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);
            CREATE INDEX IF NOT EXISTS idx_ai_ops_status ON ai_operations(status);"
        ).map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
