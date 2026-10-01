// src-tauri/src/commands/library.rs
// AI library health check: batch move/merge/tag suggestions with per-item
// confirmation, plus undo for applied operations via ai_operations rows.
use crate::ai::engine::AiEngine;
use crate::ai::organizer::LibrarySuggestion;
use crate::error::{AppError, AppResult};
use crate::notebook::service::NotebookService;
use crate::state::{lock_db, AppState};
use crate::storage::markdown::MarkdownStorage;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Compact metadata the LLM sees — no full contents, keeps prompts small.
#[tauri::command]
pub async fn ai_library_check(
    state: State<'_, Arc<AppState>>,
    seq: u32,
    locale: String,
) -> AppResult<Vec<LibrarySuggestion>> {
    let cancel = state.register_cancel(seq);
    let result = async {
        let notes_json = {
            let state = state.inner().clone();
            crate::commands::notebook::run_blocking(move || {
                let db = lock_db(&state)?;
                let notes = crate::notebook::service::NotebookService::list_recent_notes(&db, 80)?;
                let compact: Vec<serde_json::Value> = notes
                    .iter()
                    .map(|n| {
                        serde_json::json!({
                            "id": n.id,
                            "title": n.title,
                            "folder": n.folder,
                            "tags": n.tags,
                            "summary": n.summary.clone().unwrap_or_default(),
                        })
                    })
                    .collect();
                Ok(serde_json::to_string(&compact)?)
            })
            .await?
        };

        let providers = state.ai_engine.read().await.get_providers_in_order();
        if providers.is_empty() {
            return Err(AppError::AiEngine(
                "No LLM provider configured. Please add a provider in Settings.".into(),
            ));
        }
        let result = AiEngine::library_check(&providers, &notes_json, cancel, &locale).await?;
        Ok(result.suggestions)
    }
    .await;
    state.unregister_cancel(seq);
    result
}

/// Apply one suggestion with an undoable audit trail.
#[tauri::command]
pub async fn ai_library_apply(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    suggestion: LibrarySuggestion,
) -> AppResult<crate::notebook::model::Note> {
    let state = state.inner().clone();
    let kind = suggestion.kind.clone();
    let reason = suggestion.reason.clone();
    let result = crate::commands::notebook::run_blocking(
        move || -> AppResult<crate::notebook::model::Note> {
            let db = lock_db(&state)?;
            let md = MarkdownStorage::new(state.notes_dir());
            match kind.as_str() {
                "move" => {
                    let target = suggestion.target_folder.clone().ok_or_else(|| {
                        AppError::AiEngine("move suggestion missing target_folder".into())
                    })?;
                    let note = NotebookService::query_note_by_id(&db, &suggestion.note_id)?;
                    let before = serde_json::json!({ "folder": note.folder, "title": note.title });
                    let updated = NotebookService::move_note(
                        &db,
                        &md,
                        crate::notebook::model::MoveNoteRequest {
                            id: suggestion.note_id.clone(),
                            target_folder: target,
                            new_title: None,
                        },
                    )?;
                    log_undoable(&db, Some(&updated.id), "library-move", &before, &reason)?;
                    Ok(updated)
                }
                "tag" => {
                    if suggestion.tags.is_empty() {
                        return Err(AppError::AiEngine("tag suggestion missing tags".into()));
                    }
                    let note = NotebookService::query_note_by_id(&db, &suggestion.note_id)?;
                    let before = serde_json::json!({ "tags": note.tags });
                    let mut merged = note.tags.clone();
                    for tag in &suggestion.tags {
                        if !merged.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                            merged.push(tag.clone());
                        }
                    }
                    let updated = NotebookService::update_note(
                        &db,
                        &md,
                        crate::notebook::model::UpdateNoteRequest {
                            id: suggestion.note_id.clone(),
                            tags: Some(merged),
                            title: None,
                            content: None,
                            folder: None,
                        },
                    )?;
                    log_undoable(&db, Some(&updated.id), "library-tag", &before, &reason)?;
                    Ok(updated)
                }
                "merge" => {
                    let target_id = suggestion.merge_with_id.clone().ok_or_else(|| {
                        AppError::AiEngine("merge suggestion missing merge_with_id".into())
                    })?;
                    if target_id == suggestion.note_id {
                        return Err(AppError::AiEngine("merge target equals source".into()));
                    }
                    let (source, source_content) =
                        NotebookService::get_note(&db, &md, &suggestion.note_id)?;
                    let (target, target_content) = NotebookService::get_note(&db, &md, &target_id)?;
                    let separator = "\n\n---\n";
                    let appended = format!("{}{}", separator, source_content);
                    let merged_content = format!("{}{}", target_content, appended);
                    let updated = NotebookService::update_note(
                        &db,
                        &md,
                        crate::notebook::model::UpdateNoteRequest {
                            id: target_id.clone(),
                            content: Some(merged_content),
                            // union of tags
                            tags: Some({
                                let mut tags = target.tags.clone();
                                for t in &source.tags {
                                    if !tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                                        tags.push(t.clone());
                                    }
                                }
                                tags
                            }),
                            title: None,
                            folder: None,
                        },
                    )?;
                    NotebookService::trash_note(&db, &md, &source.id, &state.trash_dir())?;
                    let before = serde_json::json!({
                        "source_id": source.id,
                        "source_title": source.title,
                        "target_id": target_id,
                        "appended": appended,
                    });
                    log_undoable(&db, Some(&updated.id), "library-merge", &before, &reason)?;
                    Ok(updated)
                }
                other => Err(AppError::AiEngine(format!(
                    "unknown suggestion type: {other}"
                ))),
            }
        },
    )
    .await;
    if result.is_ok() {
        let _ = app.emit("notes-changed", "library-apply");
    }
    result
}

fn log_undoable(
    db: &crate::storage::database::Database,
    note_id: Option<&str>,
    op_type: &str,
    before: &serde_json::Value,
    reason: &str,
) -> AppResult<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    db.conn().execute(
        "INSERT INTO ai_operations (id, note_id, operation_type, before_state, after_state, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, note_id, op_type, before.to_string(), reason, "applied", now],
    )?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct AiOperationRow {
    pub id: String,
    pub note_id: Option<String>,
    pub operation_type: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn list_ai_operations(state: State<'_, Arc<AppState>>) -> AppResult<Vec<AiOperationRow>> {
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || {
        let db = lock_db(&state)?;
        let mut stmt = db.conn().prepare(
            "SELECT id, note_id, operation_type, before_state, after_state, created_at
             FROM ai_operations ORDER BY created_at DESC LIMIT 50",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AiOperationRow {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    operation_type: row.get(2)?,
                    before_state: row.get(3)?,
                    after_state: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    })
    .await
}

/// Undo an applied library operation (move / tag / merge are undoable;
/// older organize entries carry no snapshot and are rejected).
#[tauri::command]
pub async fn undo_ai_operation(
    state: State<'_, Arc<AppState>>,
    app: AppHandle,
    op_id: String,
) -> AppResult<()> {
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || {
        let db = lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());

        let row: (String, Option<String>, String, Option<String>) = db
            .conn()
            .query_row(
                "SELECT operation_type, note_id, before_state, after_state FROM ai_operations WHERE id = ?1",
                rusqlite::params![op_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|_| AppError::NotFound(format!("operation {op_id} not found")))?;
        let (op_type, note_id, before_raw, _after) = row;
        let before: serde_json::Value =
            serde_json::from_str(&before_raw).unwrap_or(serde_json::Value::Null);

        match op_type.as_str() {
            "library-move" => {
                let note_id = note_id.ok_or_else(|| AppError::AiEngine("missing note id".into()))?;
                let folder = before["folder"].as_str().unwrap_or_default().to_string();
                NotebookService::move_note(
                    &db,
                    &md,
                    crate::notebook::model::MoveNoteRequest {
                        id: note_id,
                        target_folder: folder,
                        new_title: None,
                    },
                )?;
            }
            "library-tag" => {
                let note_id = note_id.ok_or_else(|| AppError::AiEngine("missing note id".into()))?;
                let tags: Vec<String> = before["tags"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                NotebookService::update_note(
                    &db,
                    &md,
                    crate::notebook::model::UpdateNoteRequest {
                        id: note_id,
                        tags: Some(tags),
                        title: None,
                        content: None,
                        folder: None,
                    },
                )?;
            }
            "library-merge" => {
                let source_id = before["source_id"].as_str().ok_or_else(|| {
                    AppError::AiEngine("merge snapshot missing source".into())
                })?;
                let target_id = before["target_id"].as_str().ok_or_else(|| {
                    AppError::AiEngine("merge snapshot missing target".into())
                })?;
                let appended = before["appended"].as_str().unwrap_or_default();
                // Restore source from trash first.
                NotebookService::restore_note(&db, &md, source_id, &state.trash_dir())?;
                // Trim the appended block back off the target.
                let (_, target_content) = NotebookService::get_note(&db, &md, target_id)?;
                if let Some(idx) = target_content.rfind(appended) {
                    let trimmed = target_content[..idx].to_string();
                    NotebookService::update_note(
                        &db,
                        &md,
                        crate::notebook::model::UpdateNoteRequest {
                            id: target_id.to_string(),
                            content: Some(trimmed),
                            title: None,
                            folder: None,
                            tags: None,
                        },
                    )?;
                }
            }
            other => {
                return Err(AppError::AiEngine(format!("operation type '{other}' is not undoable")));
            }
        }

        db.conn()
            .execute("DELETE FROM ai_operations WHERE id = ?1", rusqlite::params![op_id])?;
        Ok(())
    })
    .await?;
    let _ = app.emit("notes-changed", "library-undo");
    Ok(())
}
