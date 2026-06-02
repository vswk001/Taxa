// src-tauri/src/ai/organizer.rs
use crate::ai::provider::{ChatOptions, ProviderConfig, create_provider};
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichResult {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub tags: Vec<String>,
}

pub struct AiOrganizer;

impl AiOrganizer {
    pub async fn process_user_input(
        config: &ProviderConfig,
        content: &str,
        folder_structure: &str,
        related_notes: &str,
    ) -> AppResult<OrganizeResult> {
        let provider = create_provider(config)?;
        let messages = PromptTemplates::categorize(content, folder_structure, related_notes);
        let response = provider.chat(messages, ChatOptions::default()).await?;

        let result: OrganizeResult = serde_json::from_str(
            response.content.trim().trim_start_matches("```json").trim_end_matches("```").trim()
        ).map_err(|e| crate::error::AppError::AiEngine(format!("Failed to parse AI response: {} - {}", e, response.content)))?;

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

        let result: EnrichResult = serde_json::from_str(
            response.content.trim().trim_start_matches("```json").trim_end_matches("```").trim()
        ).map_err(|e| crate::error::AppError::AiEngine(format!("Failed to parse enrich response: {}", e)))?;

        Ok(result)
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
