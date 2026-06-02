// src-tauri/src/error.rs
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("File I/O error: {0}")]
    FileIo(String),
    #[error("Note not found: {0}")]
    NotFound(String),
    #[error("AI engine error: {0}")]
    AiEngine(String),
    #[error("LLM provider error: {0}")]
    LlmProvider(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileIo(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}
