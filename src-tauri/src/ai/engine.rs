// src-tauri/src/ai/engine.rs
use crate::ai::organizer::{AiOrganizer, EnrichResult, OptimizeResult, OrganizeResult};
use crate::ai::provider::{
    err_if_cancelled, CancelToken, FallbackInfo, ProviderConfig, StreamCallback, StreamEvent,
};
use crate::error::{AppError, AppResult};
use crate::storage::database::Database;
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Upper bound for a single provider attempt (transport + generation).
/// Streaming clients only enforce a read timeout, so this is the hard stop.
const ATTEMPT_CEILING: Duration = Duration::from_secs(300);
/// Backoff before retrying a provider after a transient (429/5xx) failure.
const RETRY_BACKOFF: Duration = Duration::from_secs(1);

pub struct AiEngine {
    pub providers: HashMap<String, ProviderConfig>,
}

impl AiEngine {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Replace the provider set with a snapshot loaded outside (see
    /// [`load_provider_configs`]).
    pub fn set_providers(&mut self, providers: Vec<ProviderConfig>) {
        self.providers = providers.into_iter().map(|p| (p.id.clone(), p)).collect();
    }

    /// Enabled providers in fallback order: by ascending `priority`, breaking
    /// ties on default-then-name so migrated DBs (all priority 0) still keep the
    /// default provider first until the user drags to reorder.
    pub fn get_providers_in_order(&self) -> Vec<ProviderConfig> {
        let mut providers: Vec<ProviderConfig> = self
            .providers
            .values()
            .filter(|p| p.enabled)
            .cloned()
            .collect();
        providers.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.is_default.cmp(&a.is_default))
                .then_with(|| a.name.cmp(&b.name))
        });
        providers
    }

    /// Wraps an optional stream callback so we can detect whether any token
    /// was emitted for the current attempt. Before retrying after emission we
    /// send a Reset event so the UI discards the partial output.
    fn wrap_with_tracker(
        on_event: &Option<StreamCallback>,
    ) -> (Option<StreamCallback>, Arc<AtomicBool>) {
        let emitted = Arc::new(AtomicBool::new(false));
        match on_event {
            Some(cb) => {
                let user_cb = cb.clone();
                let flag = emitted.clone();
                let tracked = Arc::new(move |event: StreamEvent| {
                    if !matches!(event, StreamEvent::Fallback(_) | StreamEvent::Reset) {
                        flag.store(true, Ordering::SeqCst);
                    }
                    user_cb(event);
                }) as StreamCallback;
                (Some(tracked), emitted)
            }
            None => (None, emitted),
        }
    }

    pub async fn ask_notes(
        providers: &[ProviderConfig],
        question: &str,
        notes_context: &str,
        on_event: StreamCallback,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<String> {
        let user_on_event: Option<StreamCallback> = Some(on_event);
        Self::try_providers(providers, &user_on_event, cancel, |config, cb, cancel| {
            AiOrganizer::ask_notes(config, question, notes_context, cb, cancel, locale)
        })
        .await
    }

    pub async fn text_action(
        providers: &[ProviderConfig],
        text: &str,
        action: &str,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<String> {
        Self::try_providers(providers, &None, cancel, |config, _cb, cancel| {
            AiOrganizer::text_action(config, text, action, cancel, locale)
        })
        .await
    }

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

    fn no_providers_err() -> AppError {
        AppError::AiEngine("No LLM provider configured. Please add a provider in Settings.".into())
    }

    /// Shared provider-fallback loop. `attempt` is called per provider (plus
    /// once more for transient errors). Cancellation aborts immediately.
    /// Even when tokens were already streamed, later providers are still
    /// tried — a Reset event tells the UI to discard the partial attempt
    /// (nothing was applied to notes yet).
    async fn try_providers<F, Fut, T>(
        providers: &[ProviderConfig],
        on_event: &Option<StreamCallback>,
        cancel: CancelToken,
        mut attempt: F,
    ) -> AppResult<T>
    where
        // Takes an owned config so the returned future has no borrowed
        // arguments (async closures with references trip lifetime inference).
        F: FnMut(ProviderConfig, Option<StreamCallback>, CancelToken) -> Fut,
        Fut: Future<Output = AppResult<T>>,
    {
        if providers.is_empty() {
            return Err(Self::no_providers_err());
        }

        let mut last_err: Option<AppError> = None;
        let mut iter = providers.iter().cloned().peekable();
        while let Some(config) = iter.next() {
            err_if_cancelled(&cancel)?;
            let next_cfg = iter.peek();

            let (cb, emitted) = Self::wrap_with_tracker(on_event);
            let mut result = Self::bounded(attempt(config.clone(), cb, cancel.clone())).await;

            // One same-provider retry for transient failures (429 / 5xx).
            if let Err(e) = &result {
                if e.is_transient() {
                    tokio::time::sleep(RETRY_BACKOFF).await;
                    err_if_cancelled(&cancel)?;
                    let (cb, _) = Self::wrap_with_tracker(on_event);
                    result = Self::bounded(attempt(config.clone(), cb, cancel.clone())).await;
                }
            }

            match result {
                Ok(value) => return Ok(value),
                Err(e) => {
                    if matches!(e, AppError::Cancelled(_)) {
                        return Err(e);
                    }
                    if emitted.load(Ordering::SeqCst) {
                        if let Some(cb) = on_event {
                            cb(StreamEvent::Reset);
                        }
                    }
                    Self::notify_fallback(on_event, &config, next_cfg);
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::AiEngine("All providers failed".into())))
    }

    async fn bounded<Fut, T>(fut: Fut) -> AppResult<T>
    where
        Fut: Future<Output = AppResult<T>>,
    {
        match tokio::time::timeout(ATTEMPT_CEILING, fut).await {
            Ok(result) => result,
            Err(_) => Err(AppError::LlmProvider("Provider attempt timed out".into())),
        }
    }

    pub async fn process_input_stream(
        providers: &[ProviderConfig],
        content: &str,
        folder_structure: &str,
        related_notes: &str,
        on_event: StreamCallback,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<OrganizeResult> {
        let user_on_event: Option<StreamCallback> = Some(on_event);
        Self::try_providers(providers, &user_on_event, cancel, |config, cb, cancel| {
            AiOrganizer::process_user_input_stream(
                config,
                content,
                folder_structure,
                related_notes,
                cb,
                cancel,
                locale,
            )
        })
        .await
    }

    pub async fn enrich_note(
        providers: &[ProviderConfig],
        title: &str,
        content: &str,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<EnrichResult> {
        Self::try_providers(providers, &None, cancel, |config, _cb, cancel| {
            AiOrganizer::enrich_note(config, title, content, cancel, locale)
        })
        .await
    }

    pub async fn optimize_note(
        providers: &[ProviderConfig],
        title: &str,
        content: &str,
        instruction: &str,
        on_event: Option<StreamCallback>,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<OptimizeResult> {
        Self::try_providers(providers, &on_event, cancel, |config, cb, cancel| {
            AiOrganizer::optimize_note(config, title, content, instruction, cb, cancel, locale)
        })
        .await
    }
}

/// Load provider configs from the DB (with keyring lookup). Blocking (OS
/// keyring + SQLite) — call from `spawn_blocking`.
pub fn load_provider_configs(db: &Database) -> AppResult<Vec<ProviderConfig>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, api_url, api_key_stored, model_name, is_default, enabled, priority
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

    let mut providers = Vec::new();
    for row in rows {
        let (id, name, ptype, url, db_key, model, is_default, enabled, priority) = row?;
        // Keyring is the primary store; the DB column is the durable fallback
        // (and the migration path when the keyring service name changes).
        let api_key = crate::ai::keyring_store::KeyringStore::get_key(&id)
            .unwrap_or_else(|_| db_key.unwrap_or_default());
        providers.push(ProviderConfig {
            id,
            name,
            provider_type: ptype,
            api_url: url,
            api_key,
            model_name: model,
            is_default,
            enabled,
            priority,
        });
    }
    Ok(providers)
}
