// src-tauri/src/ai/openai.rs
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use reqwest::Client;

pub struct OpenAiProvider {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
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
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    model: String,
    usage: Option<OpenAiUsage>,
}

#[derive(serde::Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(serde::Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse> {
        let oai_messages: Vec<OpenAiMessage> = messages
            .into_iter()
            .map(|m| OpenAiMessage { role: m.role, content: m.content })
            .collect();

        let body = OpenAiRequest {
            model: if options.model.is_empty() { self.model.clone() } else { options.model },
            messages: oai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.api_url);
        let resp = self.client.post(&url)
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send().await.map_err(|e| AppError::LlmProvider(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!("OpenAI API error {}: {}", status, body)));
        }

        let data: OpenAiResponse = resp.json().await.map_err(|e| AppError::LlmProvider(e.to_string()))?;
        let content = data.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
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
