// src-tauri/src/commands/ai.rs
use crate::ai::provider::StreamEvent;
use crate::ai::organizer::OrganizeResult;
use crate::ai::organizer::OptimizeResult;
use crate::error::AppResult;
use crate::state::AppState;
use crate::storage::markdown::MarkdownStorage;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn ai_process_input(
    state: State<'_, AppState>,
    app: AppHandle,
    content: String,
    seq: u32,
) -> AppResult<OrganizeResult> {
    eprintln!("[AI] process_input START: content_len={}", content.len());

    let md = MarkdownStorage::new(state.notes_dir());

    // Scope db lock: gather data, then release before the async AI call
    let (folder_structure, related_notes) = {
        let db = state.db.lock().map_err(|e| {
            eprintln!("[AI] ERROR: db lock failed: {}", e);
            crate::error::AppError::Database(e.to_string())
        })?;
        let folders = crate::notebook::service::NotebookService::get_folder_tree(&md)
            .unwrap_or_default();
        let folder_structure = serde_json::to_string(&folders).unwrap_or_else(|_| "[]".into());

        // FTS search for semantically related notes
        let mut search_results = crate::notebook::service::NotebookService::search_notes(&db, &content, "all")
            .unwrap_or_default();

        // Also include recent notes (by title match) so LLM can decide to append
        // even when FTS doesn't find keyword overlap
        let recent_notes = crate::notebook::service::NotebookService::list_recent_notes(&db, 30)
            .unwrap_or_default();
        let existing_ids: std::collections::HashSet<String> = search_results.iter().map(|r| r.id.clone()).collect();
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

        let related_notes = serde_json::to_string(&search_results).unwrap_or_else(|_| "[]".into());
        eprintln!("[AI] gathered context: folder_data={} bytes, related={} bytes", folder_structure.len(), related_notes.len());
        (folder_structure, related_notes)
    };

    eprintln!("[AI] acquiring engine read lock...");
    let engine = state.ai_engine.read().await;
    eprintln!("[AI] engine lock acquired, calling process_input_stream...");

    // Create callback that emits Tauri events for each streaming chunk
    let app_handle = app.clone();
    let on_event = Arc::new(move |event: StreamEvent| {
        let payload = serde_json::json!({
            "seq": seq,
            "event": event,
        });
        let _ = app_handle.emit("ai-stream", payload);
    });

    let result = engine.process_input_stream(&content, &folder_structure, &related_notes, on_event).await;
    match &result {
        Ok(r) => {
            eprintln!("[AI] process_input OK: action={}, title={}", r.action, r.title);
            let json = serde_json::to_string(r).unwrap_or_else(|e| format!("serialize error: {}", e));
            eprintln!("[AI] result JSON: {} bytes", json.len());
        }
        Err(e) => eprintln!("[AI] process_input ERROR: {}", e),
    }
    result
}

#[tauri::command]
pub async fn ai_apply_result(state: State<'_, AppState>, result: OrganizeResult) -> AppResult<crate::notebook::model::Note> {
    eprintln!("[AI] apply_result: action={}, title={}, folder={}", result.action, result.title, result.folder);
    let md = MarkdownStorage::new(state.notes_dir());
    eprintln!("[AI] notes_dir={}", state.notes_dir().display());

    let note = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        match result.action.as_str() {
            "create" => crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result),
            "append" => {
                let target_id = result.target_note_id.as_deref().unwrap_or("");
                if target_id.is_empty() {
                    // Fallback: no target, create instead
                    eprintln!("[AI] append has no target_note_id, falling back to create");
                    crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result)
                } else {
                    match crate::ai::organizer::AiOrganizer::apply_append(&db, &md, target_id, &result) {
                        Ok(note) => Ok(note),
                        Err(e) => {
                            eprintln!("[AI] append failed: {}, falling back to create", e);
                            crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result)
                        }
                    }
                }
            }
            _ => Err(crate::error::AppError::AiEngine(format!("Unknown action: {}", result.action))),
        }?
    };

    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        crate::ai::organizer::AiOrganizer::log_operation(
            &db, Some(&note.id), &result.action, "",
            &serde_json::to_string(&result)?, "applied",
        )?;
    }

    eprintln!("[AI] apply_result OK: note_id={}", note.id);
    Ok(note)
}

#[tauri::command]
pub async fn ai_optimize_note(
    state: State<'_, AppState>,
    app: AppHandle,
    note_id: String,
    instruction: String,
    seq: u32,
) -> AppResult<OptimizeResult> {
    let (title, content) = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let md = MarkdownStorage::new(state.notes_dir());
        let (note, content) = crate::notebook::service::NotebookService::get_note(&db, &md, &note_id)?;
        (note.title, content)
    };

    let engine = state.ai_engine.read().await;

    let app_handle = app.clone();
    let on_event = Arc::new(move |event: StreamEvent| {
        let payload = serde_json::json!({ "seq": seq, "event": event });
        let _ = app_handle.emit("ai-stream", payload);
    });

    engine.optimize_note(&title, &content, &instruction, Some(on_event)).await
}

#[tauri::command]
pub async fn ai_enrich_note(state: State<'_, AppState>, note_id: String) -> AppResult<crate::ai::organizer::EnrichResult> {
    let (title, content) = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let md = MarkdownStorage::new(state.notes_dir());
        let (note, content) = crate::notebook::service::NotebookService::get_note(&db, &md, &note_id)?;
        (note.title, content)
    };

    let engine = state.ai_engine.read().await;
    engine.enrich_note(&title, &content).await
}

#[tauri::command]
pub async fn ai_test_provider(
    _state: State<'_, AppState>,
    provider_type: String,
    api_url: String,
    api_key: String,
    model_name: String,
) -> AppResult<bool> {
    eprintln!("[AI] test_provider: type={}, url={}, model={}", provider_type, api_url, model_name);
    let config = crate::ai::provider::ProviderConfig {
        id: "_test".into(), name: "test".into(), provider_type,
        api_url, api_key, model_name,
        is_default: false, enabled: true,
    };
    let provider = crate::ai::provider::create_provider(&config)?;
    provider.test_connection().await
}
