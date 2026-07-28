# 05 - Technical Decisions

Every dependency here was checked against its current documentation before being chosen - not assumed from training data - because package names and APIs move (e.g. `framer-motion` renamed to `motion` with a `motion/react` import path). Version numbers are what was current at decision time; pin exact versions in `Cargo.lock`/`bun.lock`, don't hand-edit them later without re-checking.

## Frontend

| Concern | Choice | Why |
| --- | --- | --- |
| Framework | React 19 | Given by the brief. |
| Animation | **`motion`** (Bun-managed), import from `motion/react` | This is the same project as Framer Motion - it renamed and moved its import path. Using the old `framer-motion` package name would install a deprecated alias. Hybrid engine (WAAPI + JS fallback) gives 120fps GPU-accelerated animation with a small footprint, which matters directly for the low-end-hardware target. |
| State | Zustand v5 | Already specified; minimal boilerplate, no `any` needed for typed stores. |
| Server-state cache | TanStack Query v5 | Already specified; used for data that's conceptually "fetched" from the Rust backend via Tauri commands, even though it's local IPC not HTTP. |
| Virtualization | `@tanstack/react-virtual` | Same ecosystem as TanStack Query, avoids a second virtualization dependency for the 100k-message list requirement. |
| Linting/formatting | **Ultracite with Oxlint + Oxfmt** | Oxc's Rust-powered Oxlint and Oxfmt provide the fast lint/format path. React-specific diagnostics are supplied through `oxlint-plugin-react-doctor` in the Ultracite Oxlint JS-plugin preset. |
| Type checking | TypeScript, `strict: true` | `noImplicitAny`, `strictNullChecks`, etc. all on - see `tsconfig.json`. |

## Backend (Rust)

| Concern | Choice | Why |
| --- | --- | --- |
| Async runtime | Tokio | Given by the brief; industry standard, what Tauri v2 itself is built on. |
| LAN discovery | `mdns-sd` | Pure-Rust mDNS/DNS-SD, no forced async runtime dependency but has a Tokio-compatible async interface, actively maintained. Preferred over `libmdns` (responder-only, no discovery) and `simple-mdns`/`rtc-mdns` (smaller communities, more manual protocol handling) for this project's need to both advertise and discover. |
| Database access | `rusqlite` (bundled SQLite) | Chosen over `sqlx` and over Tauri's official `tauri-plugin-sql`. Reasons: (1) FTS5 virtual tables and WAL pragmas are easiest to control directly against `rusqlite`'s connection API; (2) `tauri-plugin-sql` exposes a raw-SQL-execution surface to the frontend over IPC, which is a larger attack surface than needed - this project's database access goes through typed Rust query functions only, never raw SQL from the frontend; (3) per the ponytail ladder, we don't need `sqlx`'s compile-time query checking against a running DB for a single-file embedded SQLite use case. |
| Serialization / wire format | `serde` + `bincode` | Binary framing per `07_MESSAGE_PROTOCOL.md`; smaller and faster to (de)serialize than JSON for the message/gossip traffic this app generates continuously. |
| Checksums | `sha2` (SHA-256) | Standard, audited, used consistently for file-transfer integrity and any content-addressed storage. |
| TLS | `rustls` + `rcgen` (self-signed cert generation for TOFU) | `rustls` avoids a system OpenSSL dependency (relevant for the low-end/varied-OS target); `rcgen` generates the self-signed identity certs each peer needs for TOFU pinning. |
| Async TLS I/O | `tokio-rustls` | Direct Tokio stream integration for the existing `rustls` configuration; avoids introducing a second TLS stack. |
| Discovery fallback encoding | `base64` | Public certificate DER is split into bounded mDNS TXT strings; Base64 provides safe text transport without custom encoding code. |
| Error handling | `thiserror` (library crates) + `anyhow` (binary/command layer) | Standard split: typed errors in `zenpaws-*` crates, ergonomic error propagation in the thin command layer. |
| Logging | `tracing` | Structured, async-aware, works well with Tokio. |
| Linting | Clippy, `[workspace.lints.clippy]` in the workspace `Cargo.toml`: `all = "warn"`, `pedantic = "warn"`, `nursery = "warn"`, with `module_name_repetitions` and `must_use_candidate` allowed (noisy, low-value for this codebase's naming conventions). CI runs `cargo clippy --all-targets -- -D warnings`. |
| Formatting | `rustfmt`, default width, `cargo fmt --check` in CI. |
| Test runner | `cargo-nextest` (dev-only tool, not a dependency) | Faster, clearer parallel output than `cargo test`; used in CI per `20_TESTING_STRATEGY.md`. |

## Tauri plugins (official, `tauri-apps/plugins-workspace`)

Registered only as each feature phase needs them - not all up front, per the ponytail ladder's "does this need to exist yet":

| Plugin | Registered in | Purpose |
| --- | --- | --- |
| `tauri-plugin-single-instance` | Phase 1 | Prevents two ZenPaws instances presenting duplicate peer identities from the same machine. |
| `tauri-plugin-window-state` | Phase 1 | Window position/size persistence (explicit requirement). |
| `tauri-plugin-dialog` | Phase 6/7 | Native file save/open dialogs for images and file transfer. |
| `tauri-plugin-notification` | Phase 8 | Native OS notifications. |
| `tauri-plugin-autostart` | Phase 1 settings groundwork, wired to a user toggle in Phase 8 | Startup-behavior setting. |
| `tauri-plugin-clipboard-manager` | Phase 6 | Copy/paste text and images. |

`tauri-plugin-global-shortcut` was considered and deliberately **not** included - no requirement calls for a global hotkey, and adding it now would violate the ladder's first rung ("does this need to exist at all").

## Replication & conflict resolution

- **Conflict resolution: last-write-wins using Lamport logical clocks** `(peer_uuid, logical_counter)`, tie-broken by `peer_uuid` ordering. Chosen over a CRDT because messages are simple field-level edits (text, deleted flag), not concurrently-merged structured data - a CRDT would be solving a problem this app doesn't have, per the ladder. Documented in full in `27_SYNC_REPLICATION_MODEL.md`.
- **Message envelope**: 1-byte protocol version + `bincode`-serialized `serde` enum, length-prefixed (u32 LE) over TCP. See `07_MESSAGE_PROTOCOL.md`.

## Concrete limits (previously left as placeholders - now decided)

| Limit | Value |
| --- | --- |
| Max file size (any attachment) | 2 GB, chunked transfer |
| Image auto-transfer threshold | 25 MB per image before it requires manual download like other files |
| Pet state broadcast rate | 2/sec/pet (already specified in the brief) |
| SHA-256 used for | File-transfer integrity, content-addressed storage keys |

## Development toolchain and verification environment

The repository pins the local development toolchain in `.mise.toml` so the frontend and Rust checks use reproducible versions:

| Tool | Pinned version | Purpose |
| --- | --- | --- |
| mise | 2026.7.12 or newer | Installs and activates project tools |
| Bun | 1.3.14 | Package installation, scripts, Ultracite, and TypeScript checks |
| Rust | 1.97.1 | Cargo, rustfmt, Clippy, and Rust tests |

The JavaScript lockfile is `bun.lock`. Contributors use `bun install --frozen-lockfile`, not npm or another package-manager lockfile. On Debian-based Linux systems, the Tauri checks also require GTK 3, WebKitGTK 4.1, AppIndicator, librsvg, and the other packages listed in the Tauri Linux prerequisites.

The verification sequence is intentionally explicit:

```bash
mise install
bun install --frozen-lockfile
bun run verify
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

Ultracite fixes are made with `bunx ultracite fix`; the final gate is `bunx ultracite check` through `bun run check`. Rust formatting is applied with `cargo fmt` before rerunning `cargo fmt --check`. This sequence resolved the initial frontend callback, accessibility, promise-handling, and formatting diagnostics, as well as malformed Rust command-module attributes and missing Tauri icon inputs discovered during workspace compilation.

## React-specific static analysis

`oxlint-plugin-react-doctor` is an explicit development dependency used through Ultracite's Oxlint provider. The current Oxlint integration and rules reference were checked at <https://www.react.doctor/docs/configuration/eslint-and-oxlint-plugins> and <https://www.react.doctor/docs/rules>. React Doctor diagnostics remain mandatory for React, accessibility, hooks, motion, and component architecture, but there is no separate React Doctor CLI or direct script.

A React Doctor rule found that the existing Motion usage lacked reduced-motion handling. The setup and status screens now use `useReducedMotion()`, and the stylesheet includes a `prefers-reduced-motion: reduce` fallback. The React Doctor Oxlint plugin now reports no issues through `bun run check`.

## Oxc provider and installation cost

The frontend provider is now Ultracite with Oxlint + Oxfmt. Biome and the standalone React Doctor CLI are intentionally not installed or invoked. `oxlint-plugin-react-doctor` is loaded through the Ultracite Oxlint JS-plugin preset so React Doctor diagnostics remain part of the same `bun run check` command.

The project pins Node 22.18.0 in `.mise.toml` because the current Oxlint and Oxfmt TypeScript configuration loaders require Node 22.18.0 or newer. The normal workflow installs tools once with `mise install`; subsequent checks do not reinstall mise, Bun, Node, or Rust. The package scripts scope Ultracite to `src` and `vite.config.ts` so generated Rust targets, dependency trees, and unrelated repository artifacts are not scanned.

In the constrained agent sandbox, Oxlint's experimental JS-plugin allocator can abort before linting because it reserves a large fixed-size allocator. The configuration is retained for normal development machines, where the plugin is required; this environment can still validate Oxfmt and Oxlint's native rules without the plugin when diagnosing that upstream allocator limitation.

## Shadcn and chat UI integration

The frontend now has a proper shadcn/ui Vite foundation using Tailwind CSS v4, `@tailwindcss/vite`, `components.json`, the `@/components` alias, and shared `cn` utilities. The installation documentation was followed using the current registry URL without the stale `.json` suffix:

```bash
bunx shadcn@latest add "https://shadcn-collections.vercel.app/c/message" --yes
bunx shadcn@latest add "https://shadcn-collections.vercel.app/c/chat-container" --yes
```

The installed Chatcn `MessageContent` component is integrated into message rows. The Chatcn prompt-input component was intentionally not used because this application requires a contenteditable composer rather than a textarea. The project-owned `ChatcnReactionPicker` adds the LAN-specific reaction command while the installed Chatcn message component supplies the chat presentation foundation.

## Transitive GTK security advisory

The current Cargo graph includes `glib 0.18.5` through Tauri's Linux GTK3 backend. RustSec identifies the affected `VariantStrIter` implementation and lists `glib 0.20.0` as the fixed line. ZenPaws does not directly use that API; we will upgrade Tauri and the GTK3 bindings together when the compatible Tauri graph adopts the fix rather than forcing an unsafe partial override.