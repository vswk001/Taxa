// src-tauri/src/error.rs
use serde::Serialize;

#[derive(Debug, Clone, thiserror::Error)]
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
    /// HTTP 429 from the provider — retryable.
    #[error("LLM provider rate limited: {0}")]
    RateLimited(String),
    /// HTTP 5xx from the provider — retryable.
    #[error("LLM provider server error: {0}")]
    LlmServer(String),
    /// The user cancelled the operation.
    #[error("已取消")]
    Cancelled(String),
    /// The operation doesn't apply to the note's current state (e.g. it is
    /// in the trash).
    #[error("Invalid state: {0}")]
    InvalidState(String),
    /// A caller-supplied path escaped the notes vault.
    #[error("Invalid path: {0}")]
    InvalidPath(String),
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

impl AppError {
    /// True when the error is transient (rate limit / 5xx / network) and a
    /// same-provider retry is reasonable.
    pub fn is_transient(&self) -> bool {
        matches!(self, AppError::RateLimited(_) | AppError::LlmServer(_))
    }
}
