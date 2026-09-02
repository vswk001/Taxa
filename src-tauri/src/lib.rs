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
        // MUST be the first plugin: every launch reuses the single existing
        // instance instead of spawning a competing one. Without this, hidden
        // zombie instances accumulate (e.g. after an updater relaunch) and
        // both steal the webview data dir and lock taxa.exe against future
        // passive updates — the exact combination that produced the
        // "launches then vanishes" reports.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            // Close-to-tray: hide instead of exiting so the tray (and the
            // quick-capture hotkey) stay alive. Toggleable from Settings.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let state = window.app_handle().state::<Arc<AppState>>();
                    if state.close_to_tray() {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .setup(|app| {
            let data_dir = get_data_dir()?;
            // Swap in staged restore data BEFORE opening the database.
            commands::backup::complete_pending_restore(&data_dir);
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

            setup_tray(app.handle())?;

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
            commands::notebook::list_trash,
            commands::notebook::restore_note,
            commands::notebook::purge_note,
            commands::notebook::empty_trash,
            commands::notebook::get_note_links,
            commands::notebook::save_attachment,
            commands::notebook::get_vault_info,
            commands::backup::backup_vault,
            commands::backup::restore_vault,
            commands::capture::quick_capture,
            commands::capture::set_quick_capture_shortcut,
            commands::capture::hide_quick_capture,
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
            commands::ai::ai_ask_notes,
            commands::ai::ai_text_action,
            commands::settings::list_providers,
            commands::settings::save_provider,
            commands::settings::delete_provider,
            commands::settings::reorder_providers,
            commands::settings::set_close_to_tray,
            commands::graph::get_graph_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Taxa");
}

/// Tray icon + menu: keeps the app (and the global quick-capture hotkey)
/// alive after the window is closed. Quit here is a real exit.
fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let show = MenuItem::with_id(app, "show", "打开 Taxa", true, None::<&str>)?;
    let capture = MenuItem::with_id(app, "capture", "快捷捕获", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &capture, &quit])?;

    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().ok_or("no window icon")?)
        .tooltip("Taxa")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "capture" => {
                if let Some(w) = app.get_webview_window(commands::capture::QUICK_CAPTURE_WINDOW) {
                    let _ = w.show();
                    let _ = w.set_focus();
                    let _ = w.center();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    tray.set_visible(true).ok();
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
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
