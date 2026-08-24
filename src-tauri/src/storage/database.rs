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
        // Tolerate a concurrent writer (e.g. the MCP server) briefly.
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| AppError::Database(e.to_string()))?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Open an existing database without running migrations or creating it.
    /// Used by the read-only MCP server, which must not manage schema or create
    /// files. Errors if the database file does not exist.
    pub fn open_existing(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return Err(AppError::Database(format!(
                "Database not found: {}. Start the Taxa app once to initialize it.",
                path.display()
            )));
        }
        let conn = Connection::open(path)
            .map_err(|e| AppError::Database(format!("Failed to open database: {}", e)))?;
        // WAL allows concurrent readers; tolerate a writer briefly.
        let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
        Ok(Self { conn })
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
                enabled INTEGER DEFAULT 1,
                priority INTEGER DEFAULT 0
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(title, content, tags, tokenize='trigram');

            CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder);
            CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);
            CREATE INDEX IF NOT EXISTS idx_ai_ops_status ON ai_operations(status);"
        ).map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;

        // Migration: add priority column to pre-existing llm_providers tables.
        self.ensure_column("llm_providers", "priority", "INTEGER DEFAULT 0")?;

        // Migration: the key column used to be named as if it were encrypted;
        // rename it to describe what it actually holds (the keyring is primary,
        // this column is the durable fallback).
        if !self.has_column("llm_providers", "api_key_stored")?
            && self.has_column("llm_providers", "api_key_encrypted")?
        {
            self.conn
                .execute_batch(
                    "ALTER TABLE llm_providers RENAME COLUMN api_key_encrypted TO api_key_stored",
                )
                .map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;
        }

        Ok(())
    }

    /// True when the notes_fts table needs a full rebuild because it was
    /// created with the default tokenizer instead of trigram (which is what
    /// makes substring MATCH queries work for CJK text). The rebuild itself
    /// re-reads note files, so it lives in the service layer.
    pub fn fts_needs_rebuild(&self) -> AppResult<bool> {
        let sql: Option<String> = self.conn.query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='notes_fts'",
            [],
            |r| r.get(0),
        )?;
        Ok(match sql {
            None => false, // created fresh with trigram below
            Some(sql) => !sql.to_ascii_lowercase().contains("trigram"),
        })
    }

    /// Adds a column to a table if it does not already exist (idempotent).
    fn ensure_column(&self, table: &str, column: &str, decl: &str) -> AppResult<()> {
        if self.has_column(table, column)? {
            return Ok(());
        }
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, decl);
        self.conn
            .execute(&sql, [])
            .map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    /// `table` and `column` are only ever called with internal constants.
    fn has_column(&self, table: &str, column: &str) -> AppResult<bool> {
        let pragma = format!("PRAGMA table_info({})", table);
        let mut stmt = self
            .conn
            .prepare(&pragma)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
        for r in rows {
            if r? == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
