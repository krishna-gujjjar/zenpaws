# 19 - Known Issues

## Root Tauri package verification in this sandbox

The networking, shared, and settings crates pass their tests and Clippy with
warnings denied. Full root-package Tauri compilation is blocked in this Linux
sandbox because the system development package providing `gdk-3.0.pc` is not
installed. Verify the Tauri commands on a host with the Linux Tauri WebKit and
GTK prerequisites.

## Capability validation pending

Validate the generated Tauri capability schema during the first successful
`bun run tauri dev`. Pet windows are not created until Phase 4.
