//! `ZenPaws` Tauri application setup.
//!
//! This stays a thin shell: plugin registration and window/event wiring
//! only. Business logic lives in the `zenpaws-*` crates under `crates/`,
//! reached through `commands/`. See `docs/04_SYSTEM_ARCHITECTURE.md`.
//!
//! Only the plugins Phase 1 actually needs are registered here. See
//! `docs/05_TECHNICAL_DECISIONS.md`.

mod commands;

use std::sync::{Arc, Mutex};

use tauri::Manager;

/// Application-owned database state. Commands borrow it through Tauri state.
pub struct DatabaseState(pub Arc<Mutex<zenpaws_database::Database>>);

/// Runs the desktop application.
///
/// # Errors
///
/// Returns a `tauri::Error` if application setup or the runtime fails.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("zenpaws.sqlite");
            app.manage(DatabaseState(Arc::new(Mutex::new(
                zenpaws_database::Database::open(path)?,
            ))));
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::network::start_network,
            commands::network::stop_network,
            commands::pets::open_pet_window,
            commands::pets::close_pet_window,
            commands::chat::list_messages,
            commands::chat::search_messages,
            commands::chat::send_message,
            commands::chat::edit_message,
            commands::chat::delete_message
        ])
        .run(tauri::generate_context!())
}
