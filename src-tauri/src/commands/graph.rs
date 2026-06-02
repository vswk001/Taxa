// src-tauri/src/commands/graph.rs
use crate::error::AppResult;
use crate::link::graph::{build_graph, GraphData};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_graph_data(state: State<AppState>) -> AppResult<GraphData> {
    let db = state.db.lock().map_err(|e| crate::error::AppError::Database(e.to_string()))?;
    let notes_dir = state.notes_dir();
    build_graph(&db, &notes_dir)
}
