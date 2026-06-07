// src-tauri/src/ai/provider.rs
use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
        Self { model: String::new(), temperature: 0.7, max_tokens: 4096 }
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
    pub api_key: String,
    pub model_name: String,
    pub is_default: bool,
    pub enabled: bool,
}

/// Events emitted during streaming LLM responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "text")]
pub enum StreamEvent {
    /// Reasoning/thinking token delta (e.g. GLM reasoning_content)
    Reasoning(String),
    /// Content token delta
    Content(String),
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
    ) -> AppResult<ChatResponse> {
        let response = self.chat(messages, options).await?;
        if let Some(reasoning) = &response.reasoning {
            on_event(StreamEvent::Reasoning(reasoning.clone()));
        }
        on_event(StreamEvent::Content(response.content.clone()));
        Ok(response)
    }
}

pub fn build_client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(15))
        .build()
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
        _ => Err(crate::error::AppError::LlmProvider(format!(
            "Unknown provider type: {}. Supported: claude, openai, openai_compatible, glm, minimax, kimi, deepseek, custom",
            config.provider_type
        ))),
    }
}
