// src-tauri/src/commands/settings.rs
use crate::ai::provider::ProviderConfig;
use crate::commands::ai::reload_engine_providers;
use crate::commands::notebook::run_blocking;
use crate::error::AppResult;
use crate::state::{lock_db, AppState};
use rusqlite::params;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_providers(state: State<'_, Arc<AppState>>) -> AppResult<Vec<ProviderConfig>> {
    let engine = state.ai_engine.read().await;
    // Return in fallback order so the UI lists them top-to-bottom as tried.
    // API keys are never serialized across the bridge (see ProviderConfig).
    Ok(engine.get_providers_in_order())
}

#[tauri::command]
pub async fn save_provider(
    state: State<'_, Arc<AppState>>,
    config: ProviderConfig,
) -> AppResult<()> {
    let state_arc = state.inner().clone();
    // If api_key is empty, keep the existing one (the UI sends no key when
    // the user did not change it).
    let (final_config, existing_priority) = {
        let engine = state.ai_engine.read().await;
        match engine.providers.get(&config.id) {
            Some(existing) => {
                let priority = existing.priority;
                let cfg = if config.api_key.is_empty() {
                    ProviderConfig {
                        api_key: existing.api_key.clone(),
                        ..config
                    }
                } else {
                    config
                };
                (cfg, Some(priority))
            }
            None => (config, None),
        }
    };

    // Persist: keyring first (best-effort), then DB with the fallback
    // column. All blocking work on the blocking pool.
    let keyring_cfg = final_config.clone();
    run_blocking(move || {
        if !keyring_cfg.api_key.is_empty() {
            let _ = crate::ai::keyring_store::KeyringStore::save_key(
                &keyring_cfg.id,
                &keyring_cfg.api_key,
            );
        }
        Ok(())
    })
    .await?;

    // Resolve priority: keep the existing provider's priority, or append a
    // new one after the highest current priority.
    let max_priority = state
        .ai_engine
        .read()
        .await
        .providers
        .values()
        .map(|p| p.priority)
        .max()
        .unwrap_or(-1);
    let priority = existing_priority.unwrap_or(max_priority + 1);

    let cfg = final_config.clone();
    let db_state = state_arc.clone();
    run_blocking(move || {
        let db = lock_db(&db_state)?;
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO llm_providers (id, name, provider_type, api_url, api_key_stored, model_name, is_default, enabled, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                cfg.id, cfg.name, cfg.provider_type,
                cfg.api_url, cfg.api_key, cfg.model_name,
                cfg.is_default, cfg.enabled, priority,
            ],
        )?;
        if cfg.is_default {
            tx.execute("UPDATE llm_providers SET is_default = 0 WHERE id != ?1", params![cfg.id])?;
        }
        tx.commit()?;
        Ok(())
    }).await?;

    reload_engine_providers(&state_arc).await
}

/// Rewrites fallback priorities to match the given id order (first = tried first).
#[tauri::command]
pub async fn reorder_providers(
    state: State<'_, Arc<AppState>>,
    ordered_ids: Vec<String>,
) -> AppResult<()> {
    let db_state = state.inner().clone();
    let state_arc = state.inner().clone();
    run_blocking(move || {
        let db = lock_db(&db_state)?;
        let conn = db.conn();
        let tx = conn.unchecked_transaction()?;
        for (idx, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE llm_providers SET priority = ?1 WHERE id = ?2",
                params![idx as i32, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
    .await?;

    reload_engine_providers(&state_arc).await
}

#[tauri::command]
pub async fn delete_provider(state: State<'_, Arc<AppState>>, id: String) -> AppResult<()> {
    let db_state = state.inner().clone();
    run_blocking(move || {
        // Try to delete from keyring (non-fatal if the entry is absent)
        let _ = crate::ai::keyring_store::KeyringStore::delete_key(&id);
        let db = lock_db(&db_state)?;
        db.conn()
            .execute("DELETE FROM llm_providers WHERE id=?1", params![id])?;
        Ok(())
    })
    .await?;

    reload_engine_providers(&state.inner().clone()).await
}
