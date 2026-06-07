// src-tauri/src/ai/openai.rs
// OpenAI-compatible provider - works with OpenAI, GLM, MiniMax, Kimi, DeepSeek, etc.
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use futures_util::StreamExt;

pub struct OpenAiProvider {
    client: reqwest::Client,
    endpoint: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let client = build_client().unwrap_or_else(|_| reqwest::Client::new());
        let endpoint = if config.api_url.contains("/chat/completions") {
            config.api_url.clone()
        } else if config.api_url.contains("/v1") || config.api_url.contains("/v4") || config.api_url.contains("/v3") {
            format!("{}/chat/completions", config.api_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/chat/completions", config.api_url.trim_end_matches('/'))
        };
        Self { client, endpoint, api_key: config.api_key.clone(), model: config.model_name.clone() }
    }
}

#[derive(serde::Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OpenAiMessage { role: String, content: String }

#[derive(serde::Deserialize)]
struct OpenAiResponse { choices: Vec<OpenAiChoice>, model: String, usage: Option<OpenAiUsage> }

#[derive(serde::Deserialize)]
struct OpenAiChoice { message: OpenAiResponseMessage }

#[derive(serde::Deserialize)]
struct OpenAiResponseMessage {
    role: String,
    content: String,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(serde::Deserialize)]
struct OpenAiUsage { prompt_tokens: u32, completion_tokens: u32, total_tokens: u32 }

// SSE streaming chunk types
#[derive(serde::Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
    model: Option<String>,
}

#[derive(serde::Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(serde::Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse> {
        let oai_messages: Vec<OpenAiMessage> = messages
            .into_iter().map(|m| OpenAiMessage { role: m.role, content: m.content })
            .collect();

        let body = OpenAiRequest {
            model: if options.model.is_empty() { self.model.clone() } else { options.model },
            messages: oai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: false,
        };

        let resp = self.client.post(&self.endpoint)
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send().await.map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::LlmProvider(format!("API {} - {}", status, &body_text[..body_text.len().min(500)])));
        }

        let data: OpenAiResponse = serde_json::from_str(&body_text)
            .map_err(|e| AppError::LlmProvider(format!("Parse response failed: {} - {}", e, &body_text[..body_text.len().min(200)])))?;

        let choice = data.choices.first();
        let content = choice.map(|c| c.message.content.clone()).unwrap_or_default();
        let reasoning = choice.and_then(|c| c.message.reasoning_content.clone());
        Ok(ChatResponse {
            content, reasoning, model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens, completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
        on_event: StreamCallback,
    ) -> AppResult<ChatResponse> {
        let oai_messages: Vec<OpenAiMessage> = messages
            .into_iter().map(|m| OpenAiMessage { role: m.role, content: m.content })
            .collect();

        let body = OpenAiRequest {
            model: if options.model.is_empty() { self.model.clone() } else { options.model },
            messages: oai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: true,
        };

        let resp = self.client.post(&self.endpoint)
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send().await.map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AppError::LlmProvider(format!("API {} - {}", status, &body_text[..body_text.len().min(500)])));
        }

        let mut full_content = String::new();
        let mut full_reasoning = String::new();
        let mut model_name = String::new();
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::LlmProvider(format!("Stream error: {}", e)))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n\n") {
                let event_block = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                for line in event_block.lines() {
                    let data = match line.strip_prefix("data: ") {
                        Some(d) => d.trim(),
                        None => continue,
                    };
                    if data == "[DONE]" {
                        continue;
                    }

                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                        if let Some(m) = chunk.model {
                            model_name = m;
                        }
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(text) = &choice.delta.reasoning_content {
                                full_reasoning.push_str(text);
                                on_event(StreamEvent::Reasoning(text.clone()));
                            }
                            if let Some(text) = &choice.delta.content {
                                full_content.push_str(text);
                                on_event(StreamEvent::Content(text.clone()));
                            }
                        }
                    }
                }
            }
        }

        Ok(ChatResponse {
            content: full_content,
            reasoning: if full_reasoning.is_empty() { None } else { Some(full_reasoning) },
            model: model_name,
            usage: None,
        })
    }

    async fn test_connection(&self) -> AppResult<bool> {
        let resp = self.chat(
            vec![Message { role: "user".into(), content: "Say hello in one word.".into() }],
            ChatOptions { max_tokens: 64, ..Default::default() },
        ).await?;
        Ok(true)
    }
}
