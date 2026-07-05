#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;

#[cfg(target_os = "macos")]
mod keyboard;

struct TimerState {
    stop_flag: Arc<Mutex<bool>>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            stop_flag: Arc::new(Mutex::new(false)),
        }
    }
}

#[tauri::command]
fn get_cursor_position(app: tauri::AppHandle) -> Result<(f64, f64), String> {
    let pos = app.cursor_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

#[tauri::command]
fn get_window_position(app: tauri::AppHandle) -> Result<(i32, i32), String> {
    let window = app.get_webview_window("main").ok_or("no window")?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

#[tauri::command]
fn start_stretch_timer(
    app: tauri::AppHandle,
    interval_mins: u64,
    state: tauri::State<'_, TimerState>,
) {
    *state.stop_flag.lock().unwrap() = true;

    let new_flag = Arc::new(Mutex::new(false));
    *state.stop_flag.lock().unwrap() = false;

    let flag_for_thread = Arc::clone(&state.stop_flag);

    thread::spawn(move || {
        let interval = Duration::from_secs(interval_mins * 60);
        let tick = Duration::from_secs(1);
        let mut elapsed = Duration::ZERO;

        loop {
            thread::sleep(tick);
            elapsed += tick;

            if *flag_for_thread.lock().unwrap() {
                break;
            }

            if elapsed >= interval {
                elapsed = Duration::ZERO;
                let _ = app.emit("stretch-reminder", ());
            }
        }

        drop(new_flag);
    });
}

#[tauri::command]
fn stop_stretch_timer(state: tauri::State<'_, TimerState>) {
    *state.stop_flag.lock().unwrap() = true;
}

fn main() {
    tauri::Builder::default()
        .manage(TimerState::default())
        .invoke_handler(tauri::generate_handler![
            get_cursor_position,
            get_window_position,
            start_stretch_timer,
            stop_stretch_timer
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let monitor = window.current_monitor()?.unwrap();
            let screen_size = monitor.size();
            window.set_position(tauri::PhysicalPosition {
                x: (screen_size.width - 180) as i32,
                y: (screen_size.height - 220) as i32,
            })?;

            #[cfg(target_os = "macos")]
            keyboard::start(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Comnyang");
}
