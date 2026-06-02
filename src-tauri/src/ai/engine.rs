// src-tauri/src/ai/engine.rs
use crate::ai::keyring_store::KeyringStore;
use crate::ai::organizer::{AiOrganizer, OrganizeResult, EnrichResult};
use crate::ai::provider::ProviderConfig;
use crate::error::AppResult;
use crate::storage::database::Database;
use rusqlite::params;
use std::collections::HashMap;

pub struct AiEngine {
    pub providers: HashMap<String, ProviderConfig>,
}

impl AiEngine {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn load_providers(&mut self, db: &Database) -> AppResult<()> {
        let conn = db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, provider_type, api_url, model_name, is_default, enabled FROM llm_providers"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, bool>(5)?,
                row.get::<_, bool>(6)?,
            ))
        })?;

        self.providers.clear();
        for row in rows {
            let (id, name, ptype, url, model, is_default, enabled) = row?;
            let api_key = KeyringStore::get_key(&id).unwrap_or_default();
            self.providers.insert(id.clone(), ProviderConfig {
                id, name, provider_type: ptype, api_url: url,
                api_key, model_name: model, is_default, enabled,
            });
        }
        Ok(())
    }

    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.providers.values().find(|p| p.is_default && p.enabled)
    }

    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    pub async fn process_input(
        &self,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
    ) -> AppResult<OrganizeResult> {
        let config = self.get_default_provider()
            .ok_or_else(|| crate::error::AppError::AiEngine("No default LLM provider configured".into()))?;

        AiOrganizer::process_user_input(config, content, folder_structure, related_notes).await
    }

    pub async fn enrich_note(
        &self,
        title: &str,
        content: &str,
    ) -> AppResult<EnrichResult> {
        let config = self.get_default_provider()
            .ok_or_else(|| crate::error::AppError::AiEngine("No default LLM provider configured".into()))?;

        AiOrganizer::enrich_note(config, title, content).await
    }

    pub fn save_provider(&self, db: &Database, config: &ProviderConfig) -> AppResult<()> {
        KeyringStore::save_key(&config.id, &config.api_key)?;

        db.conn().execute(
            "INSERT OR REPLACE INTO llm_providers (id, name, provider_type, api_url, model_name, is_default, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![config.id, config.name, config.provider_type, config.api_url, config.model_name, config.is_default, config.enabled],
        )?;
        Ok(())
    }

    pub fn delete_provider(&self, db: &Database, id: &str) -> AppResult<()> {
        KeyringStore::delete_key(id)?;
        db.conn().execute("DELETE FROM llm_providers WHERE id=?1", params![id])?;
        Ok(())
    }
}
