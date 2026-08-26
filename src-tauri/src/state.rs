// src-tauri/src/state.rs
use crate::error::{AppError, AppResult};
use crate::storage::database::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::RwLock;

pub struct AppState {
    pub db: Mutex<Database>,
    pub data_dir: PathBuf,
    pub ai_engine: RwLock<crate::ai::engine::AiEngine>,
    /// Cancellation flags for in-flight AI requests, keyed by frontend seq.
    ai_cancels: Mutex<HashMap<u32, Arc<AtomicBool>>>,
    /// Currently registered quick-capture accelerator, if any.
    pub quick_capture_shortcut: Mutex<Option<String>>,
}

/// Lock the DB, mapping poisoning to an AppError instead of panicking.
/// Lock discipline: never hold this guard across an `.await`, and never hold
/// it while acquiring `ai_engine` (take one, drop it, then take the other).
pub fn lock_db(state: &AppState) -> AppResult<MutexGuard<'_, Database>> {
    state
        .db
        .lock()
        .map_err(|e| AppError::Database(format!("DB lock poisoned: {}", e)))
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> AppResult<Self> {
        // One-time migration from the legacy "taxis.db" file name.
        let db_path = data_dir.join("taxa.db");
        if !db_path.exists() {
            let legacy = data_dir.join("taxis.db");
            if legacy.exists() {
                let _ = std::fs::rename(&legacy, &db_path);
            }
        }
        let db = Database::new(&db_path)?;
        let ai_engine = RwLock::new(crate::ai::engine::AiEngine::new());
        Ok(Self {
            db: Mutex::new(db),
            data_dir,
            ai_engine,
            ai_cancels: Mutex::new(HashMap::new()),
            quick_capture_shortcut: Mutex::new(None),
        })
    }

    pub fn notes_dir(&self) -> PathBuf {
        self.data_dir
            .join("notebooks")
            .join("default")
            .join("notes")
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.data_dir
            .join("notebooks")
            .join("default")
            .join("attachments")
    }

    /// Soft-deleted note files live here as `<note-id>.md` until purged.
    pub fn trash_dir(&self) -> PathBuf {
        self.data_dir.join("trash")
    }

    /// Register a cancellation flag for a request sequence.
    pub fn register_cancel(&self, seq: u32) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.ai_cancels
            .lock()
            .map(|mut m| m.insert(seq, flag.clone()))
            .ok();
        flag
    }

    /// Flag a request as cancelled; the streaming loop aborts at the next
    /// chunk boundary.
    pub fn cancel_seq(&self, seq: u32) {
        if let Ok(m) = self.ai_cancels.lock() {
            if let Some(flag) = m.get(&seq) {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Remove the flag after the request finishes.
    pub fn unregister_cancel(&self, seq: u32) {
        if let Ok(mut m) = self.ai_cancels.lock() {
            m.remove(&seq);
        }
    }
}
