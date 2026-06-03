// src-tauri/src/ai/provider.rs
use crate::error::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    pub timeout_secs: u64,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: String::new(),
            temperature: 0.7,
            max_tokens: 4096,
            timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
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
    pub api_url: String,
    pub api_key: String,
    pub model_name: String,
    pub is_default: bool,
    pub enabled: bool,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse>;
    async fn test_connection(&self) -> AppResult<bool>;
}

pub fn create_provider(config: &ProviderConfig) -> AppResult<Box<dyn LlmProvider>> {
    match config.provider_type.as_str() {
        "claude" => Ok(Box::new(crate::ai::claude::ClaudeProvider::new(config))),
        "openai" | "openai_compatible" | "custom" => {
            Ok(Box::new(crate::ai::openai::OpenAiProvider::new(config)))
        }
        _ => Err(crate::error::AppError::LlmProvider(format!(
            "Unknown provider type: {}",
            config.provider_type
        ))),
    }
}
