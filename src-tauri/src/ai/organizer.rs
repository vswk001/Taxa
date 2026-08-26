// src-tauri/src/ai/organizer.rs
use crate::ai::prompt::PromptTemplates;
use crate::ai::provider::{
    create_provider, truncate_chars, CancelToken, ChatOptions, ProviderConfig, StreamCallback,
};
use crate::error::{AppError, AppResult};
use crate::notebook::model::Note;
use crate::storage::database::Database;
use crate::storage::markdown::MarkdownStorage;
use rusqlite::params;
use serde::{Deserialize, Serialize};

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
pub struct OptimizeResult {
    pub title: String,
    pub content: String,
    pub summary: String,
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

/// Format the error shown when the LLM reply is not valid JSON. Char-safe
/// truncation: byte slicing would panic on CJK text split mid-character.
fn parse_error(prefix: &str, err: &serde_json::Error, raw: &str) -> AppError {
    AppError::AiEngine(format!(
        "{}: {}. 原始: {}",
        prefix,
        err,
        truncate_chars(raw, 200)
    ))
}

pub struct AiOrganizer;

impl AiOrganizer {
    pub async fn process_user_input_stream(
        config: ProviderConfig,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
        on_event: Option<StreamCallback>,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<OrganizeResult> {
        let provider = create_provider(&config)?;
        let messages =
            PromptTemplates::categorize(content, folder_structure, related_notes, locale);

        let options = ChatOptions {
            max_tokens: 8192,
            ..ChatOptions::default()
        };

        let response = match on_event {
            Some(cb) => provider.chat_stream(messages, options, cb, cancel).await?,
            None => provider.chat(messages, options).await?,
        };

        let json_str = extract_json(&response.content);
        let mut result: OrganizeResult = serde_json::from_str(json_str)
            .map_err(|e| parse_error("AI 返回格式错误", &e, &response.content))?;
        // The folder comes straight from LLM output; the storage layer rejects
        // anything that would escape the vault, but drop obviously bogus
        // values early so users see a sane note location.
        if MarkdownStorage::validate_folder(&result.folder).is_err() {
            result.folder = String::new();
        }
        result.reasoning = response.reasoning;
        Ok(result)
    }

    pub async fn enrich_note(
        config: ProviderConfig,
        title: &str,
        content: &str,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<EnrichResult> {
        let provider = create_provider(&config)?;
        let messages = PromptTemplates::enrich(title, content, locale);
        let response = provider.chat(messages, ChatOptions::default()).await?;
        crate::ai::provider::err_if_cancelled(&cancel)?;

        let json_str = extract_json(&response.content);
        serde_json::from_str(json_str)
            .map_err(|e| parse_error("AI 返回格式错误", &e, &response.content))
    }

    pub async fn optimize_note(
        config: ProviderConfig,
        title: &str,
        content: &str,
        instruction: &str,
        on_event: Option<StreamCallback>,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<OptimizeResult> {
        let provider = create_provider(&config)?;
        let messages = PromptTemplates::optimize(title, content, instruction, locale);
        let options = ChatOptions {
            max_tokens: 8192,
            ..ChatOptions::default()
        };

        let response = match on_event {
            Some(cb) => provider.chat_stream(messages, options, cb, cancel).await?,
            None => provider.chat(messages, options).await?,
        };

        let json_str = extract_json(&response.content);
        serde_json::from_str(json_str)
            .map_err(|e| parse_error("AI 返回格式错误", &e, &response.content))
    }

    /// Q&A over notes: streams reasoning + answer, returns the raw answer
    /// text (no JSON envelope to parse).
    pub async fn ask_notes(
        config: ProviderConfig,
        question: &str,
        notes_context: &str,
        on_event: Option<StreamCallback>,
        cancel: CancelToken,
        locale: &str,
    ) -> AppResult<String> {
        let provider = create_provider(&config)?;
        let messages = PromptTemplates::ask_notes(question, notes_context, locale);
        let options = ChatOptions {
            max_tokens: 4096,
            ..ChatOptions::default()
        };
        let response = match on_event {
            Some(cb) => provider.chat_stream(messages, options, cb, cancel).await?,
            None => provider.chat(messages, options).await?,
        };
        Ok(response.content)
    }

    pub fn apply_create(
        db: &Database,
        md: &MarkdownStorage,
        result: &OrganizeResult,
    ) -> AppResult<Note> {
        crate::notebook::service::NotebookService::create_note(
            db,
            md,
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
        let (note, existing_content) =
            crate::notebook::service::NotebookService::get_note(db, md, note_id)?;
        let new_content = format!("{}\n\n---\n{}", existing_content, result.content);
        // Merge tags instead of replacing: the prompt asks the model to
        // combine them, but don't lose existing tags if it didn't.
        let mut tags = note.tags.clone();
        for t in &result.tags {
            if !tags.iter().any(|existing| existing.eq_ignore_ascii_case(t)) {
                tags.push(t.clone());
            }
        }
        crate::notebook::service::NotebookService::update_note(
            db,
            md,
            crate::notebook::model::UpdateNoteRequest {
                id: note.id.clone(),
                content: Some(new_content),
                tags: Some(tags),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_plain() {
        assert_eq!(extract_json(r#"  {"a":1}  "#), r#"{"a":1}"#);
    }

    #[test]
    fn extract_json_fenced() {
        assert_eq!(extract_json("```json\n{\"a\":1}\n```"), r#"{"a":1}"#);
        assert_eq!(extract_json("```\n{\"a\":1}\n```"), r#"{"a":1}"#);
    }

    #[test]
    fn extract_json_embedded() {
        assert_eq!(
            extract_json("Here is the result: {\"a\": {\"b\": 2}} hope it helps"),
            r#"{"a": {"b": 2}}"#
        );
    }

    #[test]
    fn extract_json_no_json_returns_trimmed() {
        assert_eq!(extract_json("  no json here  "), "no json here");
    }

    #[test]
    fn parse_error_is_char_safe_for_cjk() {
        let long_cjk = "中".repeat(500);
        let err = serde_json::from_str::<serde_json::Value>(&long_cjk).unwrap_err();
        // Must not panic despite multi-byte text.
        let e = parse_error("AI 返回格式错误", &err, &long_cjk);
        assert!(e.to_string().contains("AI 返回格式错误"));
    }
}
