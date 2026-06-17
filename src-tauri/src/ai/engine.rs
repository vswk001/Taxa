// src-tauri/src/ai/engine.rs
use crate::ai::provider::{FallbackInfo, ProviderConfig, StreamCallback, StreamEvent};
use crate::ai::organizer::{AiOrganizer, OrganizeResult, EnrichResult, OptimizeResult};
use crate::error::{AppError, AppResult};
use crate::storage::database::Database;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
            "SELECT id, name, provider_type, api_url, api_key_encrypted, model_name, is_default, enabled, priority
             FROM llm_providers WHERE enabled = 1"
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
                row.get::<_, i32>(8)?,
            ))
        })?;

        self.providers.clear();
        for row in rows {
            let (id, name, ptype, url, db_key, model, is_default, enabled, priority) = row?;
            // Try keyring first, then DB stored key
            let api_key = crate::ai::keyring_store::KeyringStore::get_key(&id)
                .unwrap_or_else(|_| db_key.unwrap_or_default());
            if api_key.is_empty() {
                eprintln!("Warning: API key for provider '{}' is empty", name);
            }
            self.providers.insert(id.clone(), ProviderConfig {
                id, name, provider_type: ptype, api_url: url,
                api_key, model_name: model, is_default, enabled, priority,
            });
        }
        Ok(())
    }

    /// Enabled providers in fallback order: by ascending `priority`, breaking
    /// ties on default-then-name so migrated DBs (all priority 0) still keep the
    /// default provider first until the user drags to reorder.
    pub fn get_providers_in_order(&self) -> Vec<ProviderConfig> {
        let mut providers: Vec<ProviderConfig> =
            self.providers.values().filter(|p| p.enabled).cloned().collect();
        providers.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| b.is_default.cmp(&a.is_default))
                .then_with(|| a.name.cmp(&b.name))
        });
        providers
    }

    /// Wraps an optional stream callback so we can detect whether any token has
    /// been emitted to the frontend. Fallback is only safe before emission;
    /// once output has started streaming, retrying would garble the UI. Fallback
    /// notifications themselves do not count as emission.
    fn wrap_with_tracker(on_event: &Option<StreamCallback>) -> (Option<StreamCallback>, Arc<AtomicBool>) {
        let emitted = Arc::new(AtomicBool::new(false));
        match on_event {
            Some(cb) => {
                let user_cb = cb.clone();
                let flag = emitted.clone();
                let tracked = Arc::new(move |event: StreamEvent| {
                    if !matches!(event, StreamEvent::Fallback(_)) {
                        flag.store(true, Ordering::SeqCst);
                    }
                    user_cb(event);
                }) as StreamCallback;
                (Some(tracked), emitted)
            }
            None => (None, emitted),
        }
    }

    pub async fn process_input_stream(
        &self,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
        on_event: StreamCallback,
        locale: &str,
    ) -> AppResult<OrganizeResult> {
        let providers = self.get_providers_in_order();
        if providers.is_empty() {
            return Err(AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ));
        }
        let user_on_event: Option<StreamCallback> = Some(on_event);

        let mut last_err: Option<AppError> = None;
        for (idx, config) in providers.iter().enumerate() {
            let (cb, emitted) = Self::wrap_with_tracker(&user_on_event);
            match AiOrganizer::process_user_input_stream(
                config, content, folder_structure, related_notes, cb, locale,
            ).await {
                Ok(result) => {
                    if idx > 0 {
                        eprintln!("[AI] Fallback succeeded with provider '{}'", config.name);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("[AI] Provider '{}' failed: {}", config.name, e);
                    last_err = Some(e);
                    if emitted.load(Ordering::SeqCst) {
                        eprintln!("[AI] Output already streaming, cannot fall back further");
                        break;
                    }
                    Self::notify_fallback(&user_on_event, config, providers.get(idx + 1));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::AiEngine("All providers failed".into())))
    }

    pub async fn enrich_note(&self, title: &str, content: &str, locale: &str) -> AppResult<EnrichResult> {
        let providers = self.get_providers_in_order();
        if providers.is_empty() {
            return Err(AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ));
        }

        let mut last_err: Option<AppError> = None;
        for (idx, config) in providers.iter().enumerate() {
            match AiOrganizer::enrich_note(config, title, content, locale).await {
                Ok(result) => {
                    if idx > 0 {
                        eprintln!("[AI] Fallback succeeded with provider '{}'", config.name);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("[AI] Provider '{}' failed: {}", config.name, e);
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::AiEngine("All providers failed".into())))
    }

    pub async fn optimize_note(
        &self,
        title: &str,
        content: &str,
        instruction: &str,
        on_event: Option<StreamCallback>,
        locale: &str,
    ) -> AppResult<OptimizeResult> {
        let providers = self.get_providers_in_order();
        if providers.is_empty() {
            return Err(AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into()
            ));
        }

        let mut last_err: Option<AppError> = None;
        for (idx, config) in providers.iter().enumerate() {
            let (cb, emitted) = Self::wrap_with_tracker(&on_event);
            match AiOrganizer::optimize_note(config, title, content, instruction, cb, locale).await {
                Ok(result) => {
                    if idx > 0 {
                        eprintln!("[AI] Fallback succeeded with provider '{}'", config.name);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("[AI] Provider '{}' failed: {}", config.name, e);
                    last_err = Some(e);
                    if emitted.load(Ordering::SeqCst) {
                        eprintln!("[AI] Output already streaming, cannot fall back further");
                        break;
                    }
                    Self::notify_fallback(&on_event, config, providers.get(idx + 1));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::AiEngine("All providers failed".into())))
    }

    /// Emits a Fallback notification (failed -> next) to the frontend, if a
    /// stream callback and a next provider exist.
    fn notify_fallback(
        on_event: &Option<StreamCallback>,
        failed: &ProviderConfig,
        next: Option<&ProviderConfig>,
    ) {
        if let (Some(cb), Some(next_cfg)) = (on_event, next) {
            cb(StreamEvent::Fallback(FallbackInfo {
                failed: failed.name.clone(),
                next: next_cfg.name.clone(),
            }));
        }
    }
}
