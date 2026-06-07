// src-tauri/src/commands/notebook.rs
use crate::error::AppResult;
use crate::notebook::model::*;
use crate::notebook::service::NotebookService;
use crate::state::AppState;
use crate::storage::markdown::MarkdownStorage;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

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
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::list_notes(&db, &md, &folder)
}

#[tauri::command]
pub fn get_folder_tree(state: State<AppState>) -> AppResult<Vec<Folder>> {
    let md = MarkdownStorage::new(state.notes_dir());
    NotebookService::get_folder_tree(&md)
}

#[tauri::command]
pub fn search_notes(state: State<AppState>, query: String, scope: Option<String>) -> AppResult<Vec<SearchResult>> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let md = MarkdownStorage::new(state.notes_dir());
    let s = scope.as_deref().unwrap_or("all");
    let mut results = NotebookService::search_notes(&db, &query, s)?;
    // Filter out phantom notes (file missing on disk)
    results.retain(|r| md.full_path(&r.path).exists());
    Ok(results)
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

#[tauri::command]
pub fn import_note(app: AppHandle) -> AppResult<Option<serde_json::Value>> {
    let path = app.dialog()
        .file()
        .set_title("导入笔记")
        .add_filter("Markdown", &["md", "txt"])
        .blocking_pick_file();

    match path {
        Some(p) => {
            let path = file_path_to_pathbuf(&p)?;
            let content = std::fs::read_to_string(&path)?;
            let title = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("导入笔记")
                .to_string();
            Ok(Some(serde_json::json!({ "title": title, "content": content })))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn export_note(app: AppHandle, title: String, content: String) -> AppResult<bool> {
    let file_name = format!("{}.md", title);
    let path = app.dialog()
        .file()
        .set_title("导出笔记")
        .add_filter("Markdown", &["md"])
        .set_file_name(&file_name)
        .blocking_save_file();

    match path {
        Some(p) => {
            let path = file_path_to_pathbuf(&p)?;
            std::fs::write(&path, &content)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[derive(serde::Serialize)]
pub struct ImportedNote {
    pub title: String,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct ImportFolderResult {
    pub folder: String,
    pub notes: Vec<ImportedNote>,
}

#[tauri::command]
pub fn import_folder(app: AppHandle) -> AppResult<Option<ImportFolderResult>> {
    let path = app.dialog()
        .file()
        .set_title("打开目录")
        .blocking_pick_folder();

    match path {
        Some(p) => {
            let dir = file_path_to_pathbuf(&p)?;
            let folder_name = dir.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("导入目录")
                .to_string();

            let mut notes = Vec::new();
            import_dir_recursive(&dir, &mut notes)?;

            Ok(Some(ImportFolderResult {
                folder: folder_name,
                notes,
            }))
        }
        None => Ok(None),
    }
}

fn import_dir_recursive(dir: &std::path::Path, notes: &mut Vec<ImportedNote>) -> AppResult<()> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                import_dir_recursive(&path, notes)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let title = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("导入笔记")
                        .to_string();
                    notes.push(ImportedNote { title, content });
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn export_folder(state: State<AppState>, app: AppHandle, folder: String) -> AppResult<bool> {
    let path = app.dialog()
        .file()
        .set_title("导出目录")
        .blocking_pick_folder();

    match path {
        Some(p) => {
            let export_dir = file_path_to_pathbuf(&p)?.join(&folder);
            std::fs::create_dir_all(&export_dir)?;

            let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
            let md = MarkdownStorage::new(state.notes_dir());

            let notes = NotebookService::list_notes(&db, &md, &folder)?;
            for note in &notes {
                if let Ok(content) = md.read_note(&note.path) {
                    let file_name = format!("{}.md", note.title);
                    std::fs::write(export_dir.join(&file_name), &content)?;
                }
            }
            Ok(true)
        }
        None => Ok(false),
    }
}

fn file_path_to_pathbuf(fp: &FilePath) -> AppResult<std::path::PathBuf> {
    match fp {
        FilePath::Path(p) => Ok(p.clone()),
        FilePath::Url(url) => url.to_file_path()
            .map_err(|_| crate::error::AppError::Config("Invalid file URL".into())),
    }
}
