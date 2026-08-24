// src-tauri/src/commands/graph.rs
use crate::error::AppResult;
use crate::link::graph::{self, GraphData};
use crate::state::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_graph_data(state: State<'_, Arc<AppState>>) -> AppResult<GraphData> {
    let state = state.inner().clone();
    crate::commands::notebook::run_blocking(move || {
        let db = crate::state::lock_db(&state)?;
        let notes_dir = state.notes_dir();
        // Pure read over notes + links; the links table is maintained
        // incrementally on note writes and reconciled once at startup.
        graph::get_graph_data(&db, &notes_dir)
    })
    .await
}
