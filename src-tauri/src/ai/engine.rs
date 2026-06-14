// src-tauri/src/ai/engine.rs
use crate::ai::provider::{ProviderConfig, StreamCallback};
use crate::ai::organizer::{AiOrganizer, OrganizeResult, EnrichResult, OptimizeResult};
use crate::error::AppResult;
use crate::storage::database::Database;
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
            "SELECT id, name, provider_type, api_url, api_key_encrypted, model_name, is_default, enabled FROM llm_providers WHERE enabled = 1"
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
            // Try keyring first, then DB stored key
            let api_key = crate::ai::keyring_store::KeyringStore::get_key(&id)
                .unwrap_or_else(|_| db_key.unwrap_or_default());
            if api_key.is_empty() {
                eprintln!("Warning: API key for provider '{}' is empty", name);
            }
            self.providers.insert(id.clone(), ProviderConfig {
                id, name, provider_type: ptype, api_url: url,
                api_key, model_name: model, is_default, enabled,
            });
        }
        Ok(())
    }

    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.providers.values().find(|p| p.is_default && p.enabled)
            .or_else(|| self.providers.values().find(|p| p.enabled))
    }

    pub async fn process_input_stream(
        &self,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
        on_event: StreamCallback,
    ) -> AppResult<OrganizeResult> {
        let config = self.get_default_provider()
            .ok_or_else(|| crate::error::AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ))?
            .clone();

        AiOrganizer::process_user_input_stream(&config, content, folder_structure, related_notes, Some(on_event)).await
    }

    pub async fn enrich_note(&self, title: &str, content: &str) -> AppResult<EnrichResult> {
        let config = self.get_default_provider()
            .ok_or_else(|| crate::error::AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ))?;

        AiOrganizer::enrich_note(config, title, content).await
    }

    pub async fn optimize_note(
        &self,
        title: &str,
        content: &str,
        instruction: &str,
        on_event: Option<StreamCallback>,
    ) -> AppResult<OptimizeResult> {
        let config = self.get_default_provider()
            .ok_or_else(|| crate::error::AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ))?
            .clone();

        AiOrganizer::optimize_note(&config, title, content, instruction, on_event).await
    }
}
