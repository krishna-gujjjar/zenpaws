# 19 - Known Issues

## Root Tauri package verification

Resolved in the current verification environment. The required Debian GTK 3, WebKitGTK 4.1, AppIndicator, librsvg, and related Tauri development packages were installed. Root Tauri compilation, workspace Clippy, formatting, and workspace tests now pass.

A fresh Linux machine still needs the platform prerequisites before running the same checks. The exact toolchain and verification commands are documented in `05_TECHNICAL_DECISIONS.md` and `22_CONTRIBUTING_GUIDE.md`.

## Capability validation pending

Validate the generated Tauri capability schema during the first successful `bun run tauri dev`. Pet windows are not created until Phase 4.

## Pet assets pending owner review

Generated pet references and sprite strips are not yet production assets. Their alpha transparency, frame alignment, actual frame counts, and provider commercial-use terms must be reviewed before `manifest.json`, `LICENSE.txt`, and asset commits are created.

## RustSec RUSTSEC-2024-0429: transitive glib warning

Cargo audit may report `glib 0.18.5` for the unsound `VariantStrIter::impl_get` implementation. The affected versions are before `glib 0.20.0`. In this project it is a transitive dependency of Tauri's Linux GTK3 backend, not a direct ZenPaws dependency, and the current Tauri/GTK3 crate graph pins the compatible `glib 0.18` line. Do not force a blind `[patch]` upgrade: the GTK3 bindings must move together and an incompatible `glib 0.20` override can break compilation.

Track the RustSec advisory and upstream gtk-rs fix. Upgrade Tauri and its GTK3 binding graph together as soon as a release pulls in the patched glib line. ZenPaws does not directly call `glib::VariantStrIter`.