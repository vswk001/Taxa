// src-tauri/src/state.rs
use crate::storage::database::Database;
use std::sync::Mutex;
use tokio::sync::RwLock;

pub struct AppState {
    pub db: Mutex<Database>,
    pub data_dir: std::path::PathBuf,
    pub ai_engine: RwLock<crate::ai::engine::AiEngine>,
}

impl AppState {
    pub fn new(data_dir: std::path::PathBuf) -> crate::error::AppResult<Self> {
        let db_path = data_dir.join("taxis.db");
        let db = Database::new(&db_path)?;
        let ai_engine = RwLock::new(crate::ai::engine::AiEngine::new());
        Ok(Self {
            db: Mutex::new(db),
            data_dir,
            ai_engine,
        })
    }

    pub fn notes_dir(&self) -> std::path::PathBuf {
        self.data_dir.join("notebooks").join("default").join("notes")
    }
}
