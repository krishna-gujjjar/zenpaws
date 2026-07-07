#![cfg(target_os = "linux")]

use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

pub fn start(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        // Best-effort Linux input event monitoring
        for i in 0..16 {
            let path = format!("/dev/input/event{}", i);
            if let Ok(mut file) = File::open(&path) {
                let handle = app_handle.clone();
                thread::spawn(move || {
                    let mut buf = [0u8; 24];
                    while let Ok(n) = file.read(&mut buf) {
                        if n >= 24 {
                            let _ = handle.emit("key-typed", ());
                        }
                    }
                });
            }
        }
    });
}
