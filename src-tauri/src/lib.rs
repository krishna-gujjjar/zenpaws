//! `ZenPaws` Tauri application setup.
//!
//! This stays a thin shell: plugin registration and window/event wiring
//! only. Business logic lives in the `zenpaws-*` crates under `crates/`,
//! reached through `commands/`. See `docs/04_SYSTEM_ARCHITECTURE.md`.
//!
//! Only the plugins Phase 1 actually needs are registered here. See
//! `docs/05_TECHNICAL_DECISIONS.md`.

mod commands;

use tauri::Manager;

/// Runs the desktop application.
///
/// # Errors
///
/// Returns a `tauri::Error` if application setup or the runtime fails.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::network::start_network,
            commands::network::stop_network
        ])
        .run(tauri::generate_context!())
}
