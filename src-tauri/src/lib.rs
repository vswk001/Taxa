// src-tauri/src/lib.rs
mod commands;
mod error;
mod state;
mod storage;
mod notebook;
mod ai;
mod link;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = get_data_dir()?;
            std::fs::create_dir_all(data_dir.join("notebooks").join("default").join("notes"))?;
            std::fs::create_dir_all(data_dir.join("notebooks").join("default").join("attachments"))?;

            let state = AppState::new(data_dir)?;
            app.manage(state);

            // Load AI providers after AppState is managed
            {
                let state_handle = app.state::<AppState>();
                let db = state_handle.db.lock().map_err(|e| error::AppError::Database(e.to_string()))?;
                state_handle.ai_engine.blocking_write().load_providers(&db)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::notebook::create_note,
            commands::notebook::get_note,
            commands::notebook::update_note,
            commands::notebook::delete_note,
            commands::notebook::move_note,
            commands::notebook::list_notes,
            commands::notebook::get_folder_tree,
            commands::notebook::search_notes,
            commands::notebook::create_folder,
            commands::notebook::rename_folder,
            commands::notebook::delete_folder,
            commands::notebook::import_note,
            commands::notebook::export_note,
            commands::notebook::import_folder,
            commands::notebook::export_folder,
            commands::ai::ai_process_input,
            commands::ai::ai_apply_result,
            commands::ai::ai_optimize_note,
            commands::ai::ai_enrich_note,
            commands::ai::ai_test_provider,
            commands::settings::list_providers,
            commands::settings::save_provider,
            commands::settings::delete_provider,
            commands::graph::get_graph_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Taxis");
}

fn get_data_dir() -> error::AppResult<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| error::AppError::Config("Cannot determine data directory".into()))?
        .join("Taxis");
    Ok(dir)
}
