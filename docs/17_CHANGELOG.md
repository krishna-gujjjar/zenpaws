# 17 - Changelog

## Phase 1 — Project Foundation (complete)

- Established the Tauri v2 + React project structure and internal Cargo
  workspace with the eight planned module crates.
- Made Bun the sole JavaScript package manager interface in metadata, scripts,
  README, and contributor instructions.
- Kept dependencies honest: removed unused opener and lint-wrapper packages,
  and removed future-phase Rust dependencies from the Phase 1 build graph.
- Added Vite Tauri development configuration, strict TypeScript, Biome,
  application shell, TanStack Query provider, and a thin Rust IPC smoke test.
- Added single-instance and window-state plugins, initial per-window
  capability files, production/development CSPs, and native icon assets.
- Created the project git repository and committed this phase as one atomic
  foundation commit.
- Restored Ultracite as the configured frontend lint and formatting wrapper.
- Restored the documented Rust minimum version of 1.97.
- Corrected the root crate Clippy findings: documentation markdown, the
  unnecessary panic in application startup, and the const command function.

## Phase 0 — Documentation and Architecture

- Added the project constitution and architecture/specification documents
  (`docs/00` through `docs/27`).
