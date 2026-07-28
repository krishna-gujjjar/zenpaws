# 21 - Deployment Guide

Placeholder-appropriate for the current stage (Phase 1 complete, no distributable build yet) - filled in with real steps starting Phase 9/10.

## Bundler targets (Tauri v2)

- **Windows**: MSI and/or NSIS installer.
- **macOS**: `.app` bundle, DMG.
- **Linux**: `.deb`, AppImage (broadest compatibility across distros).

Configured via `tauri.conf.json`'s `bundle` section - left at defaults in Phase 1, tuned per-platform in Phase 10.

## Versioning

Semantic versioning (`major.minor.patch`), starting at `0.1.0` through the phased build; `1.0.0` reserved for the first build that passes Phase 10's full testing checklist.

## Code signing

Out of scope for the phased build itself - noted here as a real requirement for any actual public distribution (unsigned installers trigger OS security warnings), to be revisited before any release beyond internal testing.

## What's deferred

Auto-update (`tauri-plugin-updater`) is not part of this build's scope - LAN peers are expected to be manually kept in sync on the same app version for now; a mismatched protocol version is detected and surfaced per `06_NETWORK_ARCHITECTURE.md` rather than silently breaking.