// src-tauri/src/commands/notebook.rs
use crate::error::AppResult;
use crate::notebook::model::*;
use crate::notebook::service::NotebookService;
use crate::state::AppState;
use crate::storage::markdown::MarkdownStorage;
use tauri::State;

#[tauri::command]
pub fn create_note(state: State<AppState>, req: CreateNoteRequest) -> AppResult<Note> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::create_note(&db, &md, req)
}

#[tauri::command]
pub fn get_note(state: State<AppState>, id: String) -> AppResult<NoteWithContent> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    let (note, content) = NotebookService::get_note(&db, &md, &id)?;
    Ok(NoteWithContent { note, content })
}

#[tauri::command]
pub fn update_note(state: State<AppState>, req: UpdateNoteRequest) -> AppResult<Note> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::update_note(&db, &md, req)
}

#[tauri::command]
pub fn delete_note(state: State<AppState>, id: String) -> AppResult<()> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::delete_note(&db, &md, &id)
}

#[tauri::command]
pub fn move_note(state: State<AppState>, req: MoveNoteRequest) -> AppResult<Note> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::move_note(&db, &md, req)
}

#[tauri::command]
pub fn list_notes(state: State<AppState>, folder: String) -> AppResult<Vec<Note>> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    NotebookService::list_notes(&db, &folder)
}

#[tauri::command]
pub fn get_folder_tree(state: State<AppState>) -> AppResult<Vec<Folder>> {
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::get_folder_tree(&md)
}

#[tauri::command]
pub fn search_notes(state: State<AppState>, query: String) -> AppResult<Vec<SearchResult>> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    NotebookService::search_notes(&db, &query)
}

#[tauri::command]
pub fn create_folder(state: State<AppState>, parent: String, name: String) -> AppResult<String> {
    let md = MarkdownStorage::new(state.notes_dir());
    md.create_folder(&parent, &name)
}

#[tauri::command]
pub fn rename_folder(state: State<AppState>, path: String, new_name: String) -> AppResult<String> {
    let md = MarkdownStorage::new(state.notes_dir());
    md.rename_folder(&path, &new_name)
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, path: String) -> AppResult<()> {
    let md = MarkdownStorage::new(state.notes_dir());
    md.delete_folder(&path)
}

#[derive(serde::Serialize)]
pub struct NoteWithContent {
    pub note: Note,
    pub content: String,
}
