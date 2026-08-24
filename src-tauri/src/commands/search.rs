// src-tauri/src/commands/search.rs
use crate::error::AppResult;
use crate::notebook::model::SearchResult;
use crate::notebook::service::NotebookService;
use crate::state::AppState;
use crate::storage::markdown::MarkdownStorage;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn search_notes(
    state: State<'_, Arc<AppState>>,
    query: String,
    scope: Option<String>,
) -> AppResult<Vec<SearchResult>> {
    let state = state.inner().clone();
    let s = scope.unwrap_or_else(|| "all".into());
    crate::commands::notebook::run_blocking(move || {
        let db = crate::state::lock_db(&state)?;
        let md = MarkdownStorage::new(state.notes_dir());
        let mut results = NotebookService::search_notes(&db, &query, &s)?;
        // Filter out phantom notes (file missing on disk)
        results.retain(|r| md.full_path(&r.path).exists());
        Ok(results)
    })
    .await
}
