// Entry point only - Tauri's convention is to keep this file untouched and
// put actual app setup in `lib.rs` so the same entry point can be shared
// with a future mobile target. See docs/04_SYSTEM_ARCHITECTURE.md.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> tauri::Result<()> {
    zenpaws_lib::run()
}
