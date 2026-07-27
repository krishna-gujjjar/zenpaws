# 19 - Known Issues

## Root Tauri package verification

Resolved in the current verification environment. The required Debian GTK 3,
WebKitGTK 4.1, AppIndicator, librsvg, and related Tauri development packages
were installed. Root Tauri compilation, workspace Clippy, formatting, and
workspace tests now pass.

A fresh Linux machine still needs the platform prerequisites before running the
same checks. The exact toolchain and verification commands are documented in
`05_TECHNICAL_DECISIONS.md` and `22_CONTRIBUTING_GUIDE.md`.

## Capability validation pending

Validate the generated Tauri capability schema during the first successful
`bun run tauri dev`. Pet windows are not created until Phase 4.

## Pet assets pending owner review

Generated pet references and sprite strips are not yet production assets. Their alpha transparency, frame alignment, actual frame counts, and provider commercial-use terms must be reviewed before `manifest.json`, `LICENSE.txt`, and asset commits are created.