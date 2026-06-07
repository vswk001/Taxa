// src-tauri/src/ai/organizer.rs
use crate::ai::provider::{ChatOptions, ProviderConfig, StreamCallback, create_provider};
use crate::ai::prompt::PromptTemplates;
use crate::error::AppResult;
use crate::notebook::model::Note;
use crate::storage::database::Database;
use crate::storage::markdown::MarkdownStorage;
use serde::{Deserialize, Serialize};
use rusqlite::params;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeResult {
    pub action: String,
    pub title: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub content: String,
    pub target_note_id: Option<String>,
    pub complexity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichResult {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub tags: Vec<String>,
}

/// Extract JSON from LLM response, handling markdown code blocks and extra text
fn extract_json(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return trimmed;
    }
    // Try ```json ... ```
    if let Some(start) = trimmed.find("```json") {
        let json_start = start + 7;
        if let Some(end) = trimmed[json_start..].find("```") {
            return trimmed[json_start..json_start + end].trim();
        }
    }
    // Try ``` ... ```
    if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        let json_start = after.find('\n').unwrap_or(0);
        let after = &after[json_start..];
        if let Some(end) = after.find("```") {
            return after[..end].trim();
        }
    }
    // Find { ... } in text
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                return &trimmed[start..=end];
            }
        }
    }
    trimmed
}

pub struct AiOrganizer;

impl AiOrganizer {
    pub async fn process_user_input(
        config: &ProviderConfig,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
    ) -> AppResult<OrganizeResult> {
        Self::process_user_input_stream(config, content, folder_structure, related_notes, None).await
    }

    pub async fn process_user_input_stream(
        config: &ProviderConfig,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
        on_event: Option<StreamCallback>,
    ) -> AppResult<OrganizeResult> {
        eprintln!("[AI] process_user_input: provider={}, model={}", config.name, config.model_name);
        let provider = create_provider(config)?;
        let messages = PromptTemplates::categorize(content, folder_structure, related_notes);
        eprintln!("[AI] Sending chat request with {} messages", messages.len());

        let response = match on_event {
            Some(cb) => provider.chat_stream(messages, ChatOptions::default(), cb).await?,
            None => provider.chat(messages, ChatOptions::default()).await?,
        };
        eprintln!("[AI] Response received, content length: {}", response.content.len());

        let json_str = extract_json(&response.content);
        eprintln!("[AI] Extracted JSON: {} bytes", json_str.len());
        let mut result: OrganizeResult = serde_json::from_str(json_str).map_err(|e| {
            crate::error::AppError::AiEngine(format!(
                "AI 返回格式错误: {}. 原始: {}", e, &response.content[..response.content.len().min(200)]
            ))
        })?;
        result.reasoning = response.reasoning;
        Ok(result)
    }

    pub async fn enrich_note(
        config: &ProviderConfig,
        title: &str,
        content: &str,
    ) -> AppResult<EnrichResult> {
        let provider = create_provider(config)?;
        let messages = PromptTemplates::enrich(title, content);
        let response = provider.chat(messages, ChatOptions::default()).await?;

        let json_str = extract_json(&response.content);
        serde_json::from_str(json_str).map_err(|e| {
            crate::error::AppError::AiEngine(format!(
                "AI 返回格式错误: {}. 原始: {}", e, &response.content[..response.content.len().min(200)]
            ))
        })
    }

    pub fn apply_create(
        db: &Database,
        md: &MarkdownStorage,
        result: &OrganizeResult,
    ) -> AppResult<Note> {
        crate::notebook::service::NotebookService::create_note(
            db, md,
            crate::notebook::model::CreateNoteRequest {
                folder: result.folder.clone(),
                title: result.title.clone(),
                content: result.content.clone(),
                tags: Some(result.tags.clone()),
            },
        )
    }

    pub fn apply_append(
        db: &Database,
        md: &MarkdownStorage,
        note_id: &str,
        result: &OrganizeResult,
    ) -> AppResult<Note> {
        let (note, existing_content) = crate::notebook::service::NotebookService::get_note(db, md, note_id)?;
        let new_content = format!("{}\n\n---\n{}", existing_content, result.content);
        crate::notebook::service::NotebookService::update_note(
            db, md,
            crate::notebook::model::UpdateNoteRequest {
                id: note.id.clone(),
                content: Some(new_content),
                tags: Some(result.tags.clone()),
                title: None,
                folder: None,
            },
        )
    }

    pub fn log_operation(
        db: &Database,
        note_id: Option<&str>,
        operation_type: &str,
        before: &str,
        after: &str,
        status: &str,
    ) -> AppResult<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        db.conn().execute(
            "INSERT INTO ai_operations (id, note_id, operation_type, before_state, after_state, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, note_id, operation_type, before, after, status, now],
        )?;
        Ok(())
    }
}
