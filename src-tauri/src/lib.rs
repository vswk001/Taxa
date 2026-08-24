mod ai;
mod commands;
pub mod error;
pub mod link;
pub mod notebook;
mod state;
pub mod storage;

use state::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let data_dir = get_data_dir()?;
            std::fs::create_dir_all(data_dir.join("notebooks").join("default").join("notes"))?;
            std::fs::create_dir_all(
                data_dir
                    .join("notebooks")
                    .join("default")
                    .join("attachments"),
            )?;

            let state = Arc::new(AppState::new(data_dir)?);
            app.manage(state.clone());

            // Load AI providers after AppState is managed
            {
                let state_handle = app.state::<Arc<AppState>>();
                let db = state::lock_db(&state_handle)?;
                let providers = ai::engine::load_provider_configs(&db)?;
                state_handle
                    .ai_engine
                    .blocking_write()
                    .set_providers(providers);
            }

            // Background maintenance: migrate a legacy FTS table to the
            // trigram tokenizer and reconcile link rows. These read every
            // note file, so they must not block startup.
            {
                let state_handle = app.state::<Arc<AppState>>();
                let state = state_handle.inner().clone();
                std::thread::spawn(move || {
                    if let Ok(db) = state::lock_db(&state) {
                        if db.fts_needs_rebuild().unwrap_or(false) {
                            let md = storage::markdown::MarkdownStorage::new(state.notes_dir());
                            let _ = notebook::service::NotebookService::rebuild_fts(&db, &md);
                        }
                        let _ = link::graph::rebuild_links(&db, &state.notes_dir());
                    }
                });
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
            commands::search::search_notes,
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
            commands::ai::ai_cancel,
            commands::settings::list_providers,
            commands::settings::save_provider,
            commands::settings::delete_provider,
            commands::settings::reorder_providers,
            commands::graph::get_graph_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Taxa");
}

fn get_data_dir() -> error::AppResult<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| error::AppError::Config("Cannot determine data directory".into()))?;
    let dir = base.join("Taxa");
    // One-time migration from the legacy "Taxis" name; falls back to the old
    // directory if the rename fails so existing data stays reachable.
    let legacy = base.join("Taxis");
    if !dir.exists() && legacy.exists() && std::fs::rename(&legacy, &dir).is_err() {
        return Ok(legacy);
    }
    Ok(dir)
}
