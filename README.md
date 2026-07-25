# ZenPaws

A LAN-only, peer-to-peer desktop messenger with interactive desktop pets,
built with Tauri v2, Rust, React 19, TypeScript, Vite, and **Bun**.

## Status

Phase 0 (architecture) is complete. Phase 1 (project foundation) has created
the Tauri/React workspace and is awaiting its first local Bun and Rust
verification. No product feature is implemented yet; see
`docs/14_DEVELOPMENT_PHASES.md` and `docs/16_PROJECT_MEMORY.md`.

## Prerequisites

- [mise](https://mise.jdx.dev/)
- Bun 1.3.14 and Rust 1.97.1, installed from `.mise.toml`
- Tauri v2 platform prerequisites: WebView2 on Windows, platform build tools
  on Linux, or Xcode Command Line Tools on macOS

Install the pinned toolchain and dependencies with:

```bash
mise install
bun install --frozen-lockfile
```

## Start development

```bash
bun run tauri dev
```

The initial lockfile must be generated once with `bun install` when Bun has
network access. Commit the resulting `bun.lock`; do not replace it with a
Node-package-manager lockfile.

## Quality gates

```bash
bun run verify
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
```

## Layout

- `src/`: React frontend
- `src-tauri/`: Tauri binary crate and its required configuration
- `src-tauri/crates/`: internal Rust library crates; implementation begins in
  the feature phase that owns each concern
- `docs/`: architecture, operating rules, and project memory