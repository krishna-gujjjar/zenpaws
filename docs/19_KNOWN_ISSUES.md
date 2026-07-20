# 19 - Known Issues

## Verification environment note

The project owner has confirmed completion of Phase 1. The local environment
still lacks the Rust toolchain, so it cannot independently rerun the root
Tauri Clippy command.

## Capability validation pending

`pet-window.json` uses the documented narrow window/event permissions, but
its exact strings must be checked against the schema emitted by the installed
Tauri version during the first local Tauri run. Pet windows are not created
until Phase 4.
