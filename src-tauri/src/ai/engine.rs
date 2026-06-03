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
            "SELECT id, name, provider_type, api_url, api_key_encrypted, model_name, is_default, enabled FROM llm_providers"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, bool>(6)?,
                row.get::<_, bool>(7)?,
            ))
        })?;

        self.providers.clear();
        for row in rows {
            let (id, name, ptype, url, db_key, model, is_default, enabled) = row?;
            // Try keyring first, then fall back to DB stored key
            let api_key = KeyringStore::get_key(&id)
                .unwrap_or_else(|_| db_key.unwrap_or_default());
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
}
