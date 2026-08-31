// src-tauri/src/commands/notebook.rs
use crate::error::{AppError, AppResult};
use crate::notebook::model::*;
use crate::notebook::service::NotebookService;
use crate::state::{lock_db, AppState};
use crate::storage::markdown::{sanitize_filename, MarkdownStorage};
use base64::Engine as _;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

// Commands are async and run their blocking work (file I/O + SQLite) on the
// blocking thread pool via spawn_blocking, keeping the main thread free for
// the webview. All DB access goes through short-lived lock_db scopes that are
// dropped before any .await.

#[tauri::command]
pub async fn create_note(
    state: State<'_, Arc<AppState>>,
    req: CreateNoteRequest,
) -> AppResult<Note> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::create_note(&db, &md, req)
    })
    .await
}

#[tauri::command]
pub async fn get_note(state: State<'_, Arc<AppState>>, id: String) -> AppResult<NoteWithContent> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        let (note, content) = NotebookService::get_note(&db, &md, &id)?;
        Ok(NoteWithContent { note, content })
    })
    .await
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, Arc<AppState>>,
    req: UpdateNoteRequest,
) -> AppResult<Note> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::update_note(&db, &md, req)
    })
    .await
}

/// Soft-delete: the note moves to the trash and stays restorable.
#[tauri::command]
pub async fn delete_note(state: State<'_, Arc<AppState>>, id: String) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::trash_note(&db, &md, &id, &state.trash_dir())
    })
    .await
}

#[tauri::command]
pub async fn list_trash(state: State<'_, Arc<AppState>>) -> AppResult<Vec<TrashItem>> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        NotebookService::list_trash(&db)
    })
    .await
}

#[tauri::command]
pub async fn restore_note(state: State<'_, Arc<AppState>>, id: String) -> AppResult<Note> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::restore_note(&db, &md, &id, &state.trash_dir())
    })
    .await
}

#[tauri::command]
pub async fn purge_note(state: State<'_, Arc<AppState>>, id: String) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        NotebookService::purge_note(&db, &id, &state.trash_dir())
    })
    .await
}

#[tauri::command]
pub async fn empty_trash(state: State<'_, Arc<AppState>>) -> AppResult<usize> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        NotebookService::empty_trash(&db, &state.trash_dir())
    })
    .await
}

/// Backlinks / outgoing links / unresolved [[targets]] for the panel.
#[tauri::command]
pub async fn get_note_links(state: State<'_, Arc<AppState>>, id: String) -> AppResult<NoteLinks> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::get_note_links(&db, &md, &id)
    })
    .await
}

#[tauri::command]
pub async fn move_note(state: State<'_, Arc<AppState>>, req: MoveNoteRequest) -> AppResult<Note> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::move_note(&db, &md, req)
    })
    .await
}

#[tauri::command]
pub async fn list_notes(state: State<'_, Arc<AppState>>, folder: String) -> AppResult<Vec<Note>> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::list_notes(&db, &md, &folder)
    })
    .await
}

#[tauri::command]
pub async fn get_folder_tree(state: State<'_, Arc<AppState>>) -> AppResult<Vec<Folder>> {
    let state = state.inner().clone();
    run_blocking(move || {
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::get_folder_tree(&md)
    })
    .await
}

#[tauri::command]
pub async fn create_folder(
    state: State<'_, Arc<AppState>>,
    parent: String,
    name: String,
) -> AppResult<String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let md = MarkdownStorage::new(state.notes_dir());
        md.create_folder(&parent, &name)
    })
    .await
}

#[tauri::command]
pub async fn rename_folder(
    state: State<'_, Arc<AppState>>,
    path: String,
    new_name: String,
) -> AppResult<String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        // Disk rename + DB row updates stay in sync.
        NotebookService::rename_folder(&db, &md, &path, &new_name)
    })
    .await
}

#[tauri::command]
pub async fn delete_folder(state: State<'_, Arc<AppState>>, path: String) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        NotebookService::delete_folder(&db, &md, &path, &state.trash_dir())
    })
    .await
}

#[derive(serde::Serialize)]
pub struct NoteWithContent {
    pub note: Note,
    pub content: String,
}

// Dialog commands: the plugin's async callback API bridged with a oneshot
// channel — never block the event loop with blocking_pick_*.

async fn pick_file(
    app: &AppHandle,
    title: &str,
    filter_name: &str,
    extensions: &[&str],
) -> AppResult<Option<FilePath>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter(filter_name, extensions)
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    rx.await
        .map_err(|_| AppError::Config("Dialog failed".into()))
}

async fn pick_folder(app: &AppHandle, title: &str) -> AppResult<Option<FilePath>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    rx.await
        .map_err(|_| AppError::Config("Dialog failed".into()))
}

async fn save_file(
    app: &AppHandle,
    title: &str,
    filter_name: &str,
    extensions: &[&str],
    file_name: &str,
) -> AppResult<Option<FilePath>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title(title)
        .add_filter(filter_name, extensions)
        .set_file_name(file_name)
        .save_file(move |path| {
            let _ = tx.send(path);
        });
    rx.await
        .map_err(|_| AppError::Config("Dialog failed".into()))
}

#[tauri::command]
pub async fn import_note(app: AppHandle) -> AppResult<Option<serde_json::Value>> {
    let path = match pick_file(&app, "导入笔记", "Markdown", &["md", "txt"]).await? {
        Some(p) => file_path_to_pathbuf(&p)?,
        None => return Ok(None),
    };
    let content = {
        let path = path.clone();
        tokio::task::spawn_blocking(move || std::fs::read_to_string(&path))
            .await
            .map_err(|e| AppError::FileIo(e.to_string()))??
    };
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("导入笔记")
        .to_string();
    Ok(Some(
        serde_json::json!({ "title": title, "content": content }),
    ))
}

#[tauri::command]
pub async fn export_note(app: AppHandle, title: String, content: String) -> AppResult<bool> {
    let file_name = format!("{}.md", sanitize_filename(&title));
    let path = match save_file(&app, "导出笔记", "Markdown", &["md"], &file_name).await? {
        Some(p) => file_path_to_pathbuf(&p)?,
        None => return Ok(false),
    };
    tokio::task::spawn_blocking(move || std::fs::write(&path, &content))
        .await
        .map_err(|e| AppError::FileIo(e.to_string()))??;
    Ok(true)
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
pub async fn import_folder(app: AppHandle) -> AppResult<Option<ImportFolderResult>> {
    let dir = match pick_folder(&app, "选择文件夹").await? {
        Some(p) => file_path_to_pathbuf(&p)?,
        None => return Ok(None),
    };
    let folder_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("导入文件夹")
        .to_string();

    let notes = tokio::task::spawn_blocking(move || {
        let mut notes = Vec::new();
        import_dir_recursive(&dir, &mut notes)?;
        Ok::<_, AppError>(notes)
    })
    .await
    .map_err(|e| AppError::FileIo(e.to_string()))??;

    Ok(Some(ImportFolderResult {
        folder: folder_name,
        notes,
    }))
}

fn import_dir_recursive(dir: &std::path::Path, notes: &mut Vec<ImportedNote>) -> AppResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        // Never follow symlinks: a cycle would recurse forever.
        if path.is_dir() && !path.is_symlink() {
            import_dir_recursive(&path, notes)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let title = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("导入笔记")
                    .to_string();
                notes.push(ImportedNote { title, content });
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn export_folder(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    folder: String,
) -> AppResult<bool> {
    let export_dir_root = match pick_folder(&app, "选择导出文件夹").await? {
        Some(p) => file_path_to_pathbuf(&p)?,
        None => return Ok(false),
    };
    let state = state.inner().clone();
    run_blocking(move || {
        let export_dir = export_dir_root.join(sanitize_filename(&folder));
        std::fs::create_dir_all(&export_dir)?;

        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        let notes = NotebookService::list_notes(&db, &md, &folder)?;

        // Collect referenced attachments while writing the notes so the
        // exported folder is self-contained (markdown references stay valid
        // on the target machine).
        let mut referenced: Vec<String> = Vec::new();
        let re =
            regex::Regex::new(r#"attachments/([^\s\)\]"']+)"#).expect("invalid attachment regex");
        for note in &notes {
            if let Ok(content) = md.read_note(&note.path) {
                for cap in re.captures_iter(&content) {
                    if let Some(name) = cap.get(1) {
                        let name = name.as_str().to_string();
                        if !referenced.contains(&name) {
                            referenced.push(name);
                        }
                    }
                }
                let file_name = format!("{}.md", sanitize_filename(&note.title));
                std::fs::write(export_dir.join(&file_name), &content)?;
            }
        }

        let attachments_src = state.attachments_dir();
        let attachments_dst = export_dir.join("attachments");
        for name in &referenced {
            let src = attachments_src.join(name);
            if src.exists() {
                std::fs::create_dir_all(&attachments_dst)?;
                let _ = std::fs::copy(&src, attachments_dst.join(name));
            }
        }
        Ok(true)
    })
    .await
}

/// Allowed attachment extensions (images only for v1).
const ATTACHMENT_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "avif"];

/// Save an attachment (base64) under the notebook's attachments dir and
/// return the vault-relative path to embed in markdown.
#[tauri::command]
pub async fn save_attachment(
    state: State<'_, Arc<AppState>>,
    file_name: String,
    data: String,
) -> AppResult<String> {
    let state = state.inner().clone();
    run_blocking(move || {
        let ext = file_name
            .rsplit('.')
            .next()
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        if !ATTACHMENT_EXTS.contains(&ext.as_str()) {
            return Err(AppError::InvalidPath(format!(
                "unsupported attachment type: .{ext}"
            )));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data.as_bytes())
            .map_err(|e| AppError::Serialization(format!("invalid base64: {e}")))?;
        let attachments = state.attachments_dir();
        std::fs::create_dir_all(&attachments)?;
        let name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        std::fs::write(attachments.join(&name), bytes)?;
        Ok(format!("attachments/{name}"))
    })
    .await
}

/// Absolute paths the frontend needs (asset URLs for images).
#[derive(serde::Serialize)]
pub struct VaultInfo {
    pub notes_dir: String,
    pub attachments_dir: String,
    pub trash_dir: String,
}

#[tauri::command]
pub async fn get_vault_info(state: State<'_, Arc<AppState>>) -> AppResult<VaultInfo> {
    let state = state.inner().clone();
    run_blocking(move || {
        Ok(VaultInfo {
            notes_dir: state.notes_dir().to_string_lossy().to_string(),
            attachments_dir: state.attachments_dir().to_string_lossy().to_string(),
            trash_dir: state.trash_dir().to_string_lossy().to_string(),
        })
    })
    .await
}

fn file_path_to_pathbuf(fp: &FilePath) -> AppResult<std::path::PathBuf> {
    match fp {
        FilePath::Path(p) => Ok(p.clone()),
        FilePath::Url(url) => url
            .to_file_path()
            .map_err(|_| AppError::Config("Invalid file URL".into())),
    }
}

/// Run a blocking closure on the blocking thread pool and flatten the join
/// result into AppResult.
pub(crate) async fn run_blocking<T, F>(f: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> AppResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::AiEngine(format!("Background task failed: {}", e)))?
}
