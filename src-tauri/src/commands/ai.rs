// src-tauri/src/commands/ai.rs
use crate::ai::organizer::OrganizeResult;
use crate::error::AppResult;
use crate::state::AppState;
use crate::storage::markdown::MarkdownStorage;
use tauri::State;

#[tauri::command]
pub async fn ai_process_input(state: State<'_, AppState>, content: String) -> AppResult<OrganizeResult> {
    let md = MarkdownStorage::new(state.notes_dir());

    // Get the data we need synchronously
    let (folder_structure, related_notes) = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let folders = crate::notebook::service::NotebookService::get_folder_tree(&md)?;
        let folder_structure = serde_json::to_string(&folders)?;
        let search_results = crate::notebook::service::NotebookService::search_notes(&db, &content)?;
        let related_notes = serde_json::to_string(&search_results)?;
        (folder_structure, related_notes)
    };

    // Now do the async AI processing
    let engine = state.ai_engine.read().await;
    engine.process_input(&content, &folder_structure, &related_notes).await
}

#[tauri::command]
pub async fn ai_apply_result(state: State<'_, AppState>, result: OrganizeResult) -> AppResult<crate::notebook::model::Note> {
    let md = MarkdownStorage::new(state.notes_dir());

    let note = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;

        match result.action.as_str() {
            "create" => crate::ai::organizer::AiOrganizer::apply_create(&db, &md, &result)?,
            "append" => {
                let target_id = result.target_note_id.as_deref().unwrap_or("");
                crate::ai::organizer::AiOrganizer::apply_append(&db, &md, target_id, &result)?
            }
            _ => return Err(crate::error::AppError::AiEngine(format!("Unknown action: {}", result.action))),
        }
    };

    // Log operation in a separate scope
    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        crate::ai::organizer::AiOrganizer::log_operation(
            &db, Some(&note.id), &result.action, "", &serde_json::to_string(&result)?, "applied"
        )?;
    }

    Ok(note)
}

#[tauri::command]
pub async fn ai_enrich_note(state: State<'_, AppState>, note_id: String) -> AppResult<crate::ai::organizer::EnrichResult> {
    // Get the note data synchronously
    let (title, content) = {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        let md = MarkdownStorage::new(state.notes_dir());
        let (note, content) = crate::notebook::service::NotebookService::get_note(&db, &md, &note_id)?;
        (note.title, content)
    };

    // Now do the async AI processing
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
    let config = crate::ai::provider::ProviderConfig {
        id: "_test".into(), name: "test".into(), provider_type,
        api_url, api_key, model_name,
        is_default: false, enabled: true,
    };
    let provider = crate::ai::provider::create_provider(&config)?;
    provider.test_connection().await
}
