// src-tauri/src/ai/claude.rs
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;

pub struct ClaudeProvider {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let client = build_client().unwrap_or_else(|_| reqwest::Client::new());
        // api_url should be the full endpoint, e.g. "https://api.anthropic.com/v1/messages"
        let endpoint = if config.api_url.contains("/v1/messages") {
            config.api_url.clone()
        } else {
            format!("{}/v1/messages", config.api_url.trim_end_matches('/'))
        };
        Self { client, endpoint, api_key: config.api_key.clone(), model: config.model_name.clone() }
    }
}

#[derive(serde::Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudeMessage { role: String, content: String }

#[derive(serde::Deserialize)]
struct ClaudeResponse { content: Vec<ClaudeContent>, model: String, usage: Option<ClaudeUsage> }

#[derive(serde::Deserialize)]
struct ClaudeContent { text: String }

#[derive(serde::Deserialize)]
struct ClaudeUsage { input_tokens: u32, output_tokens: u32 }

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse> {
        let system_msg = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());
        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter().filter(|m| m.role != "system")
            .map(|m| ClaudeMessage { role: m.role, content: m.content })
            .collect();

        let body = ClaudeRequest {
            model: if options.model.is_empty() { self.model.clone() } else { options.model },
            max_tokens: options.max_tokens,
            messages: claude_messages,
            system: system_msg,
            temperature: Some(options.temperature),
        };

        let resp = self.client.post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send().await.map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::LlmProvider(format!("Claude API {} - {}", status, &body_text[..body_text.len().min(500)])));
        }

        let data: ClaudeResponse = serde_json::from_str(&body_text)
            .map_err(|e| AppError::LlmProvider(format!("Parse response failed: {} - {}", e, &body_text[..body_text.len().min(200)])))?;

        Ok(ChatResponse {
            content: data.content.into_iter().map(|c| c.text).collect::<Vec<_>>().join(""),
            reasoning: None,
            model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens, completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
            }),
        })
    }

    async fn test_connection(&self) -> AppResult<bool> {
        let _ = self.chat(
            vec![Message { role: "user".into(), content: "Say hello in one word.".into() }],
            ChatOptions { max_tokens: 64, ..Default::default() },
        ).await?;
        Ok(true)
    }
}
