#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
fn get_cursor_position(app: tauri::AppHandle) -> Result<(f64, f64), String> {
    let pos = app.cursor_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_cursor_position])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let monitor = window.current_monitor()?.unwrap();
            let screen_size = monitor.size();
            window.set_position(tauri::PhysicalPosition {
                x: (screen_size.width - 180) as i32,
                y: (screen_size.height - 220) as i32,
            })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Comnyang");
}
