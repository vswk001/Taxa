// src-tauri/src/ai/openai.rs
// OpenAI-compatible provider - works with OpenAI, GLM, MiniMax, Kimi, DeepSeek, etc.
use crate::ai::provider::*;
use crate::error::{AppError, AppResult};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

pub struct OpenAiProvider {
    endpoint: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            endpoint: endpoint_url(&config.api_url),
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
    #[serde(skip_serializing_if = "std::ops::Not::not")]
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
    message: OpenAiResponseMessage,
}

#[derive(serde::Deserialize)]
struct OpenAiResponseMessage {
    content: String,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(serde::Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn endpoint_url(api_url: &str) -> String {
    if api_url.contains("/chat/completions") {
        api_url.to_string()
    } else if api_url.contains("/v1") || api_url.contains("/v4") || api_url.contains("/v3") {
        format!("{}/chat/completions", api_url.trim_end_matches('/'))
    } else {
        format!("{}/v1/chat/completions", api_url.trim_end_matches('/'))
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> AppResult<ChatResponse> {
        let oai_messages: Vec<OpenAiMessage> = messages
            .into_iter()
            .map(|m| OpenAiMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let body = OpenAiRequest {
            model: if options.model.is_empty() {
                self.model.clone()
            } else {
                options.model
            },
            messages: oai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: false,
        };

        let resp = chat_client()
            .post(&self.endpoint)
            .header("authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::LlmProvider(format!("Network error: {}", e)))?;

        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(http_error(status, &body_text));
        }

        let data: OpenAiResponse = serde_json::from_str(&body_text).map_err(|e| {
            AppError::LlmProvider(format!(
                "Parse response failed: {} - {}",
                e,
                truncate_chars(&body_text, 200)
            ))
        })?;

        let choice = data.choices.first();
        let content = choice
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        let reasoning = choice.and_then(|c| c.message.reasoning_content.clone());
        Ok(ChatResponse {
            content,
            reasoning,
            model: data.model,
            usage: data.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
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
        let oai_messages: Vec<OpenAiMessage> = messages
            .into_iter()
            .map(|m| OpenAiMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let body = OpenAiRequest {
            model: if options.model.is_empty() {
                self.model.clone()
            } else {
                options.model
            },
            messages: oai_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            stream: true,
        };

        let resp = stream_client()
            .post(&self.endpoint)
            .header("authorization", format!("Bearer {}", self.api_key))
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
        let mut truncated = false;

        // eventsource-stream parses SSE at the byte level: multibyte UTF-8
        // split across network chunks, CRLF line endings, and trailing
        // partial events are all handled per spec.
        let mut stream = resp.bytes_stream().eventsource();
        while let Some(event) = stream.next().await {
            err_if_cancelled(&cancel)?;
            let event = event.map_err(|e| AppError::LlmProvider(format!("Stream error: {}", e)))?;
            let data = event.data.trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }

            let value: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue, // keep-alive/comment payloads
            };
            if let Some(err) = value.get("error") {
                let message = err
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(AppError::LlmProvider(format!(
                    "Stream aborted by provider: {}",
                    message
                )));
            }
            if let Some(m) = value.get("model").and_then(|m| m.as_str()) {
                if !m.is_empty() {
                    model_name = m.to_string();
                }
            }
            if let Some(choice) = value.get("choices").and_then(|c| c.get(0)) {
                if let Some(delta) = choice.get("delta") {
                    if let Some(text) = delta.get("reasoning_content").and_then(|t| t.as_str()) {
                        full_reasoning.push_str(text);
                        on_event(StreamEvent::Reasoning(text.to_string()));
                    }
                    if let Some(text) = delta.get("content").and_then(|t| t.as_str()) {
                        full_content.push_str(text);
                        on_event(StreamEvent::Content(text.to_string()));
                    }
                }
                if let Some(reason) = choice.get("finish_reason").and_then(|r| r.as_str()) {
                    if reason == "length" {
                        truncated = true;
                    }
                }
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
            usage: None,
        })
    }

    async fn test_connection(&self) -> AppResult<bool> {
        self.chat(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_url_variants() {
        assert_eq!(
            endpoint_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_url("https://api.openai.com/v1/chat/completions"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_url("https://example.com"),
            "https://example.com/v1/chat/completions"
        );
        assert_eq!(
            endpoint_url("https://example.com/v4/"),
            "https://example.com/v4/chat/completions"
        );
    }
}
