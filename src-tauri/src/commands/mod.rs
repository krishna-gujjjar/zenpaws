//! Thin Tauri command handlers - IPC translation only.
//!
//! Every command here validates input, delegates to the relevant
//! `zenpaws-*` crate, and maps its result to a serializable type. See
//! `docs/04_SYSTEM_ARCHITECTURE.md` and `docs/15_AI_AGENT_RULES.md`.

pub mod chat;
pub mod network;
pub mod pets;

use serde::Serialize;

/// Phase 1 smoke-test command verifying the frontend↔backend IPC bridge
/// compiles and works end-to-end. Not a feature - remove or repurpose once
/// Phase 2 adds the first real command and this is no longer needed as a
/// wiring check.
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub name: &'static str,
    pub version: &'static str,
}

#[tauri::command]
pub const fn app_status() -> AppStatus {
    AppStatus {
        name: "ZenPaws",
        version: env!("CARGO_PKG_VERSION"),
    }
}
