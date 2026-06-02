// src-tauri/src/ai/claude.rs
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;

pub struct ClaudeProvider {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            client: Client::new(),
            api_url: config.api_url.trim_end_matches('/').to_string(),
            api_key: config.api_key.clone(),
            model: config.model_name.clone(),
        }
    }
}

#[derive(serde::Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    model: String,
    usage: Option<ClaudeUsage>,
}

#[derive(serde::Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(serde::Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse> {
        let system_msg = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone());

        let claude_messages: Vec<ClaudeMessage> = messages
            .into_iter()
            .filter(|m| m.role != "system")
            .map(|m| ClaudeMessage { role: m.role, content: m.content })
            .collect();

        let body = ClaudeRequest {
            model: if options.model.is_empty() { self.model.clone() } else { options.model },
            max_tokens: options.max_tokens,
            messages: claude_messages,
            temperature: Some(options.temperature),
        };

        let url = format!("{}/v1/messages", self.api_url);
        let mut req = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body);

        if let Some(system) = system_msg {
            req = req.header("anthropic-system", &system);
        }

        let resp = req.send().await.map_err(|e| AppError::LlmProvider(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!("Claude API error {}: {}", status, body)));
        }

        let data: ClaudeResponse = resp.json().await.map_err(|e| AppError::LlmProvider(e.to_string()))?;
        Ok(ChatResponse {
            content: data.content.into_iter().map(|c| c.text).collect(),
            model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
            }),
        })
    }

    async fn test_connection(&self) -> AppResult<bool> {
        let resp = self.chat(
            vec![Message { role: "user".into(), content: "Hi".into() }],
            ChatOptions { max_tokens: 10, ..Default::default() },
        ).await?;
        Ok(!resp.content.is_empty())
    }
}
