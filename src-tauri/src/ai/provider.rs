// src-tauri/src/ai/provider.rs
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: String::new(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub reasoning: Option<String>,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    /// Full API endpoint URL, e.g. "https://api.openai.com/v1/chat/completions"
    pub api_url: String,
    /// Never serialized back across the IPC bridge — the frontend keeps keys
    /// out of its state and sends a key only when the user changes it.
    #[serde(skip_serializing, default)]
    pub api_key: String,
    pub model_name: String,
    pub is_default: bool,
    pub enabled: bool,
    /// Fallback order: lower is tried first. Set via drag-and-drop reordering.
    #[serde(default)]
    pub priority: i32,
}

/// Cooperative cancellation flag for in-flight LLM requests.
pub type CancelToken = Arc<AtomicBool>;

pub fn cancelled(cancel: &CancelToken) -> bool {
    cancel.load(Ordering::Relaxed)
}

pub fn err_if_cancelled(cancel: &CancelToken) -> AppResult<()> {
    if cancelled(cancel) {
        Err(AppError::Cancelled("Cancelled by user".into()))
    } else {
        Ok(())
    }
}

/// Payload for a provider-fallback notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackInfo {
    /// Name of the provider that just failed.
    pub failed: String,
    /// Name of the next provider being tried.
    pub next: String,
}

/// Events emitted during streaming LLM responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "text")]
pub enum StreamEvent {
    /// Reasoning/thinking token delta (e.g. GLM reasoning_content)
    Reasoning(String),
    /// Content token delta
    Content(String),
    /// A provider failed and the engine is falling back to the next one.
    Fallback(FallbackInfo),
    /// Previously streamed output belongs to a failed attempt and is about to
    /// be replaced by the next provider's attempt — the UI should clear it.
    Reset,
}

pub type StreamCallback = Arc<dyn Fn(StreamEvent) + Send + Sync>;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse>;
    async fn test_connection(&self) -> AppResult<bool>;

    /// Streaming variant. Default falls back to non-streaming `chat`.
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
        on_event: StreamCallback,
        cancel: CancelToken,
    ) -> AppResult<ChatResponse> {
        err_if_cancelled(&cancel)?;
        let response = self.chat(messages, options).await?;
        if let Some(reasoning) = &response.reasoning {
            on_event(StreamEvent::Reasoning(reasoning.clone()));
        }
        on_event(StreamEvent::Content(response.content.clone()));
        Ok(response)
    }
}

fn fallback_client() -> reqwest::Client {
    reqwest::Client::new()
}

/// Client for non-streaming requests: bounded by a total timeout.
pub fn chat_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| fallback_client())
    })
}

/// Client for streaming requests: no total timeout (long streams are legal);
/// a 60s read timeout detects a stalled connection instead.
pub fn stream_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| fallback_client())
    })
}

/// Build the error for a non-2xx HTTP response, classifying retryable
/// statuses (429/5xx) so the engine can retry the same provider.
pub fn http_error(status: reqwest::StatusCode, body: &str) -> AppError {
    let snippet = truncate_chars(body, 500);
    match status.as_u16() {
        429 => AppError::RateLimited(format!("HTTP 429 - {}", snippet)),
        s if (500..600).contains(&s) => AppError::LlmServer(format!("HTTP {} - {}", s, snippet)),
        _ => AppError::LlmProvider(format!("HTTP {} - {}", status, snippet)),
    }
}

/// Char-boundary-safe truncation. Byte slicing (`&s[..n]`) panics when n
/// lands inside a multi-byte character — the common case for CJK text.
pub fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Create the appropriate LLM provider. All providers use their own API format.
/// The api_url must be the full endpoint URL.
pub fn create_provider(config: &ProviderConfig) -> AppResult<Box<dyn LlmProvider>> {
    match config.provider_type.as_str() {
        "claude" => Ok(Box::new(crate::ai::claude::ClaudeProvider::new(config))),
        // All OpenAI-compatible providers (OpenAI, GLM, MiniMax, Kimi, DeepSeek, etc.)
        "openai" | "openai_compatible" | "custom" | "glm" | "minimax" | "kimi" | "deepseek" => {
            Ok(Box::new(crate::ai::openai::OpenAiProvider::new(config)))
        }
        _ => Err(AppError::LlmProvider(format!(
            "Unknown provider type: {}. Supported: claude, openai, openai_compatible, glm, minimax, kimi, deepseek, custom",
            config.provider_type
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_never_panics_on_cjk() {
        let s = "中文内容比较长的时候";
        assert_eq!(truncate_chars(s, 3), "中文内");
        assert_eq!(truncate_chars(s, 0), "");
    }

    #[test]
    fn provider_config_hides_api_key_in_json() {
        let config = ProviderConfig {
            id: "p1".into(),
            name: "test".into(),
            provider_type: "openai".into(),
            api_url: "https://x".into(),
            api_key: "secret".into(),
            model_name: "m".into(),
            is_default: false,
            enabled: true,
            priority: 0,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret"));
        // Deserialization still accepts the key when the frontend sends one.
        let with_key = r#"{"id":"p1","name":"test","provider_type":"openai","api_url":"https://x","api_key":"k","model_name":"m","is_default":false,"enabled":true,"priority":0}"#;
        let parsed: ProviderConfig = serde_json::from_str(with_key).unwrap();
        assert_eq!(parsed.api_key, "k");
        // And tolerates its absence.
        let without: ProviderConfig = serde_json::from_str(
            r#"{"id":"p1","name":"test","provider_type":"openai","api_url":"https://x","model_name":"m","is_default":false,"enabled":true,"priority":0}"#,
        ).unwrap();
        assert_eq!(without.api_key, "");
    }
}
