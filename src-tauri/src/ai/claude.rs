// src-tauri/src/ai/claude.rs
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

pub struct ClaudeProvider {
    endpoint: String,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        // api_url should be the full endpoint, e.g. "https://api.anthropic.com/v1/messages"
        let endpoint = if config.api_url.contains("/v1/messages") {
            config.api_url.clone()
        } else {
            format!("{}/v1/messages", config.api_url.trim_end_matches('/'))
        };
        Self {
            endpoint,
            api_key: config.api_key.clone(),
            model: config.model_name.clone(),
        }
    }

    fn split_messages(messages: Vec<Message>) -> (Option<String>, Vec<ClaudeMessage>) {
        let system_msg = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());
        let rest = messages
            .into_iter()
            .filter(|m| m.role != "system")
            .map(|m| ClaudeMessage {
                role: m.role,
                content: m.content,
            })
            .collect();
        (system_msg, rest)
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
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
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
    /// Non-text blocks (thinking, tool_use) have no text; default keeps the
    /// whole response parseable when extended thinking is on.
    #[serde(default)]
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
        let (system_msg, claude_messages) = Self::split_messages(messages);

        let body = ClaudeRequest {
            model: if options.model.is_empty() {
                self.model.clone()
            } else {
                options.model
            },
            max_tokens: options.max_tokens,
            messages: claude_messages,
            system: system_msg,
            temperature: Some(options.temperature),
            stream: false,
        };

        let resp = chat_client()
            .post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(http_error(status, &body_text));
        }

        let data: ClaudeResponse = serde_json::from_str(&body_text).map_err(|e| {
            AppError::LlmProvider(format!(
                "Parse response failed: {} - {}",
                e,
                truncate_chars(&body_text, 200)
            ))
        })?;

        Ok(ChatResponse {
            content: data
                .content
                .into_iter()
                .map(|c| c.text)
                .collect::<Vec<_>>()
                .join(""),
            reasoning: None,
            model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
            }),
        })
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
        on_event: StreamCallback,
        cancel: CancelToken,
    ) -> AppResult<ChatResponse> {
        let (system_msg, claude_messages) = Self::split_messages(messages);

        let body = ClaudeRequest {
            model: if options.model.is_empty() {
                self.model.clone()
            } else {
                options.model
            },
            max_tokens: options.max_tokens,
            messages: claude_messages,
            system: system_msg,
            temperature: Some(options.temperature),
            stream: true,
        };

        let resp = stream_client()
            .post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(http_error(status, &body_text));
        }

        let mut full_content = String::new();
        let mut full_reasoning = String::new();
        let mut model_name = String::new();
        let mut usage: Option<TokenUsage> = None;
        let mut truncated = false;

        let mut stream = resp.bytes_stream().eventsource();
        while let Some(event) = stream.next().await {
            err_if_cancelled(&cancel)?;
            let event = event.map_err(|e| AppError::LlmProvider(format!("Stream error: {}", e)))?;
            let data = event.data.trim();
            if data.is_empty() {
                continue;
            }
            let value: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            match value.get("type").and_then(|t| t.as_str()).unwrap_or("") {
                "error" => {
                    let message = value
                        .pointer("/error/message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error");
                    return Err(AppError::LlmProvider(format!(
                        "Stream aborted by provider: {}",
                        message
                    )));
                }
                "message_start" => {
                    if let Some(m) = value.pointer("/message/model").and_then(|m| m.as_str()) {
                        model_name = m.to_string();
                    }
                    if let Some(u) = value.pointer("/message/usage") {
                        usage = Some(parse_usage(u));
                    }
                }
                "content_block_delta" => {
                    if let Some(delta) = value.get("delta") {
                        if let Some(text) = delta.get("thinking").and_then(|t| t.as_str()) {
                            full_reasoning.push_str(text);
                            on_event(StreamEvent::Reasoning(text.to_string()));
                        }
                        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                            full_content.push_str(text);
                            on_event(StreamEvent::Content(text.to_string()));
                        }
                    }
                }
                "message_delta" => {
                    if let Some(u) = value.get("usage") {
                        usage = Some(parse_usage(u));
                    }
                    if value.pointer("/delta/stop_reason").and_then(|r| r.as_str())
                        == Some("max_tokens")
                    {
                        truncated = true;
                    }
                }
                _ => {}
            }
        }

        if truncated {
            return Err(AppError::LlmProvider(
                "Response truncated: hit max_tokens limit before completing the answer".into(),
            ));
        }

        Ok(ChatResponse {
            content: full_content,
            reasoning: if full_reasoning.is_empty() {
                None
            } else {
                Some(full_reasoning)
            },
            model: model_name,
            usage,
        })
    }

    async fn test_connection(&self) -> AppResult<bool> {
        let _ = self
            .chat(
                vec![Message {
                    role: "user".into(),
                    content: "Say hello in one word.".into(),
                }],
                ChatOptions {
                    max_tokens: 64,
                    ..Default::default()
                },
            )
            .await?;
        Ok(true)
    }
}

fn parse_usage(u: &serde_json::Value) -> TokenUsage {
    let input = u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let output = u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    TokenUsage {
        prompt_tokens: input,
        completion_tokens: output,
        total_tokens: input + output,
    }
}
