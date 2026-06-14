// src-tauri/src/commands/settings.rs
use crate::ai::provider::ProviderConfig;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;
use rusqlite::params;

#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> AppResult<Vec<ProviderConfig>> {
    let engine = state.ai_engine.read().await;
    // Return in fallback order so the UI lists them top-to-bottom as tried.
    Ok(engine.get_providers_in_order())
}

#[tauri::command]
pub async fn save_provider(state: State<'_, AppState>, config: ProviderConfig) -> AppResult<()> {
    // If api_key is empty, try to keep the existing one
    let final_config = if config.api_key.is_empty() {
        let engine = state.ai_engine.read().await;
        if let Some(existing) = engine.providers.get(&config.id) {
            ProviderConfig { api_key: existing.api_key.clone(), ..config }
        } else {
            config
        }
    } else {
        // Try to save new API key to keyring (non-fatal if it fails)
        let _ = crate::ai::keyring_store::KeyringStore::save_key(&config.id, &config.api_key);
        config
    };

    // Resolve priority: keep the existing provider's priority, or append a new
    // one after the highest current priority.
    let priority = {
        let engine = state.ai_engine.read().await;
        match engine.providers.get(&final_config.id) {
            Some(existing) => existing.priority,
            None => engine.providers.values().map(|p| p.priority).max().unwrap_or(-1) + 1,
        }
    };

    // Save provider + API key to database
    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        db.conn().execute(
            "INSERT OR REPLACE INTO llm_providers (id, name, provider_type, api_url, api_key_encrypted, model_name, is_default, enabled, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                final_config.id, final_config.name, final_config.provider_type,
                final_config.api_url, final_config.api_key, final_config.model_name,
                final_config.is_default, final_config.enabled, priority,
            ],
        )?;
    }

    // If this provider is set as default, unset all others
    if final_config.is_default {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        db.conn().execute(
            "UPDATE llm_providers SET is_default = 0 WHERE id != ?1",
            params![final_config.id],
        )?;
    }

    // Reload providers into engine
    let mut engine = state.ai_engine.write().await;
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    engine.load_providers(&db)?;
    Ok(())
}

/// Rewrites fallback priorities to match the given id order (first = tried first).
#[tauri::command]
pub async fn reorder_providers(state: State<'_, AppState>, ordered_ids: Vec<String>) -> AppResult<()> {
    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        for (idx, id) in ordered_ids.iter().enumerate() {
            db.conn().execute(
                "UPDATE llm_providers SET priority = ?1 WHERE id = ?2",
                params![idx as i32, id],
            )?;
        }
    }

    let mut engine = state.ai_engine.write().await;
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    engine.load_providers(&db)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, AppState>, id: String) -> AppResult<()> {
    // Try to delete from keyring (non-fatal)
    let _ = crate::ai::keyring_store::KeyringStore::delete_key(&id);

    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        db.conn().execute("DELETE FROM llm_providers WHERE id=?1", params![id])?;
    }

    let mut engine = state.ai_engine.write().await;
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    engine.load_providers(&db)?;
    Ok(())
}
