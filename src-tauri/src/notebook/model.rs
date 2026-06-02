// src-tauri/src/notebook/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub path: String,
    pub title: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub word_count: i64,
    pub summary: Option<String>,
    pub ai_categorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
    pub path: String,
    pub children: Vec<Folder>,
    pub note_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub folder: String,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub folder: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveNoteRequest {
    pub id: String,
    pub target_folder: String,
    pub new_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub parent: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameFolderRequest {
    pub path: String,
    pub new_name: String,
}
