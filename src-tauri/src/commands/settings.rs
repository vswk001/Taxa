// src-tauri/src/commands/settings.rs
use crate::ai::provider::ProviderConfig;
use crate::ai::keyring_store::KeyringStore;
use crate::error::AppResult;
use crate::state::AppState;
use tauri::State;
use rusqlite::params;

#[tauri::command]
pub async fn list_providers(state: State<'_, AppState>) -> AppResult<Vec<ProviderConfig>> {
    let engine = state.ai_engine.read().await;
    Ok(engine.providers.values().cloned().collect())
}

#[tauri::command]
pub async fn save_provider(state: State<'_, AppState>, config: ProviderConfig) -> AppResult<()> {
    // Save API key to keyring
    KeyringStore::save_key(&config.id, &config.api_key)?;

    // Save provider to database (synchronously)
    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        db.conn().execute(
            "INSERT OR REPLACE INTO llm_providers (id, name, provider_type, api_url, model_name, is_default, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![config.id, config.name, config.provider_type, config.api_url, config.model_name, config.is_default, config.enabled],
        )?;
        // db lock is released here
    }

    // Reload providers into engine (async)
    let mut engine = state.ai_engine.write().await;
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    engine.load_providers(&db)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, AppState>, id: String) -> AppResult<()> {
    // Delete API key from keyring
    KeyringStore::delete_key(&id)?;

    // Delete provider from database (synchronously)
    {
        let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
        db.conn().execute("DELETE FROM llm_providers WHERE id=?1", params![id])?;
        // db lock is released here
    }

    // Reload providers into engine (async)
    let mut engine = state.ai_engine.write().await;
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    engine.load_providers(&db)?;
    Ok(())
}
