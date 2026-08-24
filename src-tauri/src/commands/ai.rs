// src-tauri/src/commands/ai.rs
use crate::ai::engine::{self, AiEngine};
use crate::ai::organizer::{OptimizeResult, OrganizeResult};
use crate::ai::provider::StreamEvent;
use crate::error::AppResult;
use crate::state::{lock_db, AppState};
use crate::storage::markdown::MarkdownStorage;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

// Lock discipline: the engine's RwLock is taken only to clone the provider
// snapshot, and the DB mutex only inside spawn_blocking — never held together
// and never across the LLM await, so Settings stays responsive during
// generation.

#[tauri::command]
pub async fn ai_process_input(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    content: String,
    seq: u32,
    locale: String,
) -> AppResult<OrganizeResult> {
    let cancel = state.register_cancel(seq);
    let result = ai_process_input_inner(&state, &app, &content, seq, &locale, cancel.clone()).await;
    state.unregister_cancel(seq);
    result
}

async fn ai_process_input_inner(
    state: &Arc<AppState>,
    app: &AppHandle,
    content: &str,
    seq: u32,
    locale: &str,
    cancel: crate::ai::provider::CancelToken,
) -> AppResult<OrganizeResult> {
    let md = MarkdownStorage::new(state.notes_dir());

    // Gather context on the blocking pool, then release the DB lock before
    // the async AI call.
    let (folder_structure, related_notes) = {
        let state = state.clone();
        let content = content.to_string();
        crate::commands::notebook::run_blocking(move || {
            let db = lock_db(&state)?;
            let folders =
                crate::notebook::service::NotebookService::get_folder_tree(&md).unwrap_or_default();
            let folder_structure = serde_json::to_string(&folders).unwrap_or_else(|_| "[]".into());

            // FTS search for semantically related notes
            let mut search_results =
                crate::notebook::service::NotebookService::search_notes(&db, &content, "all")
                    .unwrap_or_default();

            // Also include recent notes (by title match) so the LLM can decide
            // to append even without keyword overlap
            let recent_notes =
                crate::notebook::service::NotebookService::list_recent_notes(&db, 30)
                    .unwrap_or_default();
            let existing_ids: std::collections::HashSet<String> =
                search_results.iter().map(|r| r.id.clone()).collect();
            for note in recent_notes {
                if !existing_ids.contains(&note.id) {
                    search_results.push(crate::notebook::model::SearchResult {
                        id: note.id,
                        title: note.title,
                        path: note.path,
                        snippet: String::new(),
                        rank: 99.0,
                    });
                }
            }

            // Filter out phantom notes (in DB but file missing on disk)
            search_results.retain(|r| md.full_path(&r.path).exists());

            let related_notes =
                serde_json::to_string(&search_results).unwrap_or_else(|_| "[]".into());
            Ok((folder_structure, related_notes))
        })
        .await?
    };

    // Clone the provider snapshot, then drop the read lock before awaiting
    // the LLM (the engine lock must not span the request).
    let providers = state.ai_engine.read().await.get_providers_in_order();

    let app_handle = app.clone();
    let on_event = Arc::new(move |event: StreamEvent| {
        let _ = app_handle.emit(
            "ai-stream",
            serde_json::json!({ "seq": seq, "event": event }),
        );
    });

    AiEngine::process_input_stream(
        &providers,
        content,
        &folder_structure,
        &related_notes,
        on_event,
        cancel,
        locale,
    )
    .await
}

/// Abort an in-flight AI request (user pressed stop / the UI timed out).
#[tauri::command]
pub async fn ai_cancel(state: State<'_, Arc<AppState>>, seq: u32) -> AppResult<()> {
    state.cancel_seq(seq);
    Ok(())
}

#[tauri::command]
pub async fn ai_apply_result(
    state: State<'_, Arc<AppState>>,
    result: OrganizeResult,
) -> AppResult<crate::notebook::model::Note> {
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || {
        let md = MarkdownStorage::new(state.notes_dir());
        let note = {
            let db = lock_db(&state)?;
            match result.action.as_str() {
                "create" => crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result),
                "append" => {
                    let target_id = result.target_note_id.as_deref().unwrap_or("");
                    if target_id.is_empty() {
                        Err(crate::error::AppError::AiEngine(
                            "AI chose to append but did not specify a target note".into(),
                        ))
                    } else {
                        match crate::ai::organizer::AiOrganizer::apply_append(
                            &db, &md, target_id, &result,
                        ) {
                            // Target vanished between generation and apply:
                            // creating a fresh note is the sensible recovery.
                            Err(crate::error::AppError::NotFound(_)) => {
                                crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result)
                            }
                            other => other,
                        }
                    }
                }
                _ => Err(crate::error::AppError::AiEngine(format!(
                    "Unknown action: {}",
                    result.action
                ))),
            }?
        };

        {
            let db = lock_db(&state)?;
            crate::ai::organizer::AiOrganizer::log_operation(
                &db,
                Some(&note.id),
                &result.action,
                "",
                &serde_json::to_string(&result)?,
                "applied",
            )?;
        }
        Ok(note)
    })
    .await
}

#[tauri::command]
pub async fn ai_optimize_note(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    note_id: String,
    instruction: String,
    seq: u32,
    locale: String,
) -> AppResult<OptimizeResult> {
    let cancel = state.register_cancel(seq);
    let result = ai_optimize_note_inner(
        &state,
        &app,
        &note_id,
        &instruction,
        seq,
        &locale,
        cancel.clone(),
    )
    .await;
    state.unregister_cancel(seq);
    result
}

async fn ai_optimize_note_inner(
    state: &Arc<AppState>,
    app: &AppHandle,
    note_id: &str,
    instruction: &str,
    seq: u32,
    locale: &str,
    cancel: crate::ai::provider::CancelToken,
) -> AppResult<OptimizeResult> {
    let (title, content) = {
        let state = state.clone();
        let note_id = note_id.to_string();
        crate::commands::notebook::run_blocking(move || {
            let db = lock_db(&state)?;
            let md = MarkdownStorage::new(state.notes_dir());
            let (note, content) =
                crate::notebook::service::NotebookService::get_note(&db, &md, &note_id)?;
            Ok((note.title, content))
        })
        .await?
    };

    let providers = state.ai_engine.read().await.get_providers_in_order();

    let app_handle = app.clone();
    let on_event = Arc::new(move |event: StreamEvent| {
        let _ = app_handle.emit(
            "ai-stream",
            serde_json::json!({ "seq": seq, "event": event }),
        );
    });

    AiEngine::optimize_note(
        &providers,
        &title,
        &content,
        instruction,
        Some(on_event),
        cancel,
        locale,
    )
    .await
}

#[tauri::command]
pub async fn ai_enrich_note(
    state: State<'_, Arc<AppState>>,
    note_id: String,
    locale: String,
) -> AppResult<crate::ai::organizer::EnrichResult> {
    let cancel = state.register_cancel(u32::MAX);
    let result = {
        let (title, content) = {
            let state = state.inner().clone();
            let note_id = note_id.clone();
            crate::commands::notebook::run_blocking(move || {
                let db = lock_db(&state)?;
                let md = MarkdownStorage::new(state.notes_dir());
                let (note, content) =
                    crate::notebook::service::NotebookService::get_note(&db, &md, &note_id)?;
                Ok((note.title, content))
            })
            .await?
        };

        let providers = state.ai_engine.read().await.get_providers_in_order();
        AiEngine::enrich_note(&providers, &title, &content, cancel.clone(), &locale).await
    };
    state.unregister_cancel(u32::MAX);
    result
}

#[tauri::command]
pub async fn ai_test_provider(
    _state: State<'_, Arc<AppState>>,
    provider_type: String,
    api_url: String,
    api_key: String,
    model_name: String,
) -> AppResult<bool> {
    let config = crate::ai::provider::ProviderConfig {
        id: "_test".into(),
        name: "test".into(),
        provider_type,
        api_url,
        api_key,
        model_name,
        is_default: false,
        enabled: true,
        priority: 0,
    };
    let provider = crate::ai::provider::create_provider(&config)?;
    provider.test_connection().await
}

/// Reload providers from the DB into the engine (DB read on the blocking
/// pool; the engine write lock is taken only to swap the snapshot).
pub(crate) async fn reload_engine_providers(state: &Arc<AppState>) -> AppResult<()> {
    let providers = {
        let state = state.clone();
        crate::commands::notebook::run_blocking(move || {
            let db = lock_db(&state)?;
            engine::load_provider_configs(&db)
        })
        .await?
    };
    state.ai_engine.write().await.set_providers(providers);
    Ok(())
}
