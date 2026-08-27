// src-tauri/src/commands/capture.rs
// Global quick capture: a hotkey summoning a small always-on-top input
// window whose submissions either go through the AI organize pipeline
// (auto-applied when the result is simple) or fall back to an Inbox note.
use crate::ai::engine::AiEngine;
use crate::ai::organizer::AiOrganizer;
use crate::ai::provider::{err_if_cancelled, CancelToken, StreamEvent};
use crate::error::AppResult;
use crate::state::{lock_db, AppState};
use crate::storage::markdown::MarkdownStorage;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub const QUICK_CAPTURE_WINDOW: &str = "quick-capture";

/// Register (or disable) the global shortcut that toggles the quick-capture
/// window. Passing an empty accelerator unregisters.
#[tauri::command]
pub async fn set_quick_capture_shortcut(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    accelerator: String,
) -> AppResult<()> {
    let shortcuts = app.global_shortcut();
    // Unregister whatever we registered before.
    if let Ok(current) = state.quick_capture_shortcut.lock() {
        if let Some(old) = current.as_deref() {
            let _ = shortcuts.unregister(old);
        }
    }
    if accelerator.is_empty() {
        set_stored_shortcut(&state, None);
        return Ok(());
    }
    let acc = accelerator.clone();
    shortcuts
        .on_shortcut(accelerator.as_str(), move |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_quick_capture(app);
            }
        })
        .map_err(|e| {
            crate::error::AppError::Config(format!("failed to register shortcut {acc}: {e}"))
        })?;
    set_stored_shortcut(&state, Some(accelerator));
    Ok(())
}

fn set_stored_shortcut(state: &AppState, value: Option<String>) {
    if let Ok(mut stored) = state.quick_capture_shortcut.lock() {
        *stored = value;
    }
}

fn toggle_quick_capture(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_WINDOW) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.center();
        }
    }
}

/// Hide the quick-capture window (called by its Esc handler / after submit).
#[tauri::command]
pub async fn hide_quick_capture(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(QUICK_CAPTURE_WINDOW) {
        let _ = window.hide();
    }
    Ok(())
}

/// Handle a quick-capture submission. Runs the AI organize pipeline headless
/// and auto-applies simple results; anything else (no providers, errors,
/// complex results) lands as a raw note in the Inbox folder so nothing is
/// ever lost. Returns the note id so the capture window can show feedback.
#[tauri::command]
pub async fn quick_capture(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    content: String,
    locale: String,
) -> AppResult<crate::notebook::model::Note> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err(crate::error::AppError::AiEngine("empty capture".into()));
    }

    let providers = state.ai_engine.read().await.get_providers_in_order();
    if !providers.is_empty() {
        let md = MarkdownStorage::new(state.notes_dir());
        // Gather the same context the sidebar organize flow uses.
        let context = {
            let state = state.inner().clone();
            let content_for_ctx = content.clone();
            crate::commands::notebook::run_blocking(move || {
                let db = lock_db(&state)?;
                let folders = crate::notebook::service::NotebookService::get_folder_tree(&md)
                    .unwrap_or_default();
                let folder_structure =
                    serde_json::to_string(&folders).unwrap_or_else(|_| "[]".into());
                let mut related = crate::notebook::service::NotebookService::search_notes(
                    &db,
                    &content_for_ctx,
                    "all",
                )
                .unwrap_or_default();
                related.retain(|r| md.full_path(&r.path).exists());
                related.truncate(8);
                let related_json = serde_json::to_string(&related).unwrap_or_else(|_| "[]".into());
                Ok((folder_structure, related_json))
            })
            .await
        };

        if let Ok((folder_structure, related_json)) = context {
            let cancel: CancelToken = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let noop = Arc::new(|_event: StreamEvent| {});
            // A capture must feel instant: bound the whole AI attempt so a
            // slow or unreachable provider falls back to the Inbox quickly
            // instead of hanging the capture window for minutes.
            let attempt = AiEngine::process_input_stream(
                &providers,
                &content,
                &folder_structure,
                &related_json,
                noop,
                cancel.clone(),
                &locale,
            );
            let result = tokio::time::timeout(std::time::Duration::from_secs(60), attempt)
                .await
                .ok(); // a timeout falls through to the Inbox fallback
            if err_if_cancelled(&cancel).is_err() {
                // fall through to inbox
            } else if let Some(Ok(result)) = result {
                if result.complexity == "simple" {
                    let state = state.inner().clone();
                    let applied = crate::commands::notebook::run_blocking(move || {
                        let db = lock_db(&state)?;
                        let md = MarkdownStorage::new(state.notes_dir());
                        match result.action.as_str() {
                            "append" => {
                                let target = result.target_note_id.as_deref().unwrap_or("");
                                if target.is_empty() {
                                    AiOrganizer::apply_create(&db, &md, &result)
                                } else {
                                    AiOrganizer::apply_append(&db, &md, target, &result)
                                        .or_else(|_| AiOrganizer::apply_create(&db, &md, &result))
                                }
                            }
                            _ => AiOrganizer::apply_create(&db, &md, &result),
                        }
                    })
                    .await;
                    if let Ok(note) = applied {
                        let _ = app.emit("notes-changed", &note.id);
                        return Ok(note);
                    }
                }
            }
        }
    }

    // Fallback (and complex results): keep the raw text in the Inbox.
    let title = content
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.chars().take(50).collect::<String>())
        .unwrap_or_else(|| "Capture".into());
    let state = state.inner().clone();
    let note = crate::commands::notebook::run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        crate::notebook::service::NotebookService::create_note(
            &db,
            &md,
            crate::notebook::model::CreateNoteRequest {
                folder: "Inbox".into(),
                title,
                content,
                tags: Some(vec!["inbox".into()]),
            },
        )
    })
    .await?;
    let _ = app.emit("notes-changed", &note.id);
    Ok(note)
}
