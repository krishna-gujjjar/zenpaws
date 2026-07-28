# 17 - Changelog

## Phase 4 - Pet Engine

- Added data-driven pet manifest and registry validation, local state and cursor behavior, discrete sync event handling, and transparent pet-window commands.
- Added the pet asset prompt kit and generated review-pending sprite references and strips outside the phase commit.

## Phase 3 - Database Foundation

- Added bundled SQLite through `rusqlite`, WAL configuration, migrations, FTS5, typed settings, keyset pagination, and disk-backed migration verification.
- Added SQLite-backed TOFU certificate storage and wired it into the network runtime.

## Phase 2 - Networking Foundation

- Implemented mDNS discovery with bounded UDP broadcast fallback.
- Implemented versioned binary frames, encrypted TCP, post-TLS UUID and username handshake, heartbeat state transitions, and bounded reconnect backoff.
- Implemented persistent local TLS identity and JSON-backed TOFU peer certificate pin storage.
- Added Tauri start and stop network commands, inbound accept lifecycle, and mDNS/UDP outbound lifecycle.
- Recorded DM-only read receipt semantics.

## Phase 1 — Project Foundation (complete)

- Established the Tauri v2 + React project structure and internal Cargo workspace with the eight planned module crates.
- Made Bun the sole JavaScript package manager interface in metadata, scripts, README, and contributor instructions.
- Kept dependencies honest: removed unused opener and lint-wrapper packages, and removed future-phase Rust dependencies from the Phase 1 build graph.
- Added Vite Tauri development configuration, strict TypeScript, the frontend lint/format pipeline, application shell, TanStack Query provider, and a thin Rust IPC smoke test.
- Added single-instance and window-state plugins, initial per-window capability files, production/development CSPs, and native icon assets.
- Created the project git repository and committed this phase as one atomic foundation commit.
- Restored Ultracite as the configured frontend lint and formatting wrapper.
- Restored the documented Rust minimum version of 1.97.
- Corrected the root crate Clippy findings: documentation markdown, the unnecessary panic in application startup, and the const command function.

## Phase 0 — Documentation and Architecture

- Added the project constitution and architecture/specification documents (`docs/00` through `docs/27`).

## Verification environment and lint remediation

- Added `.mise.toml` pinning Bun 1.3.14 and Rust 1.97.1, installed mise, and generated the canonical `bun.lock` with Bun.
- Fixed Ultracite diagnostics in the chat UI by stabilizing JSX callbacks, handling asynchronous promises without `void`, correcting ARIA semantics, simplifying nested conditionals, and applying formatter output.
- Fixed malformed Rust command-module attributes and applied `cargo fmt` formatting.
- Added valid RGBA Tauri icon inputs required by `tauri::generate_context!`.
- Installed the Debian GTK/WebKitGTK/AppIndicator development dependencies that were preventing root Tauri compilation.
- Verified `bun run verify`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --workspace` successfully.

## React Doctor and file-size compliance

- Added `oxlint-plugin-react-doctor` to the Ultracite Oxlint JS-plugin configuration and made its diagnostics part of `bun run check`.
- Documented the React Doctor Oxlint integration in `15_AI_AGENT_RULES.md` and its dependency decision in `05_TECHNICAL_DECISIONS.md`.
- Added reduced-motion handling with `useReducedMotion()` and a CSS media-query fallback after the React Doctor plugin identified the missing accessibility behavior.
- Split the chat command types and network service connection, lifecycle, and error modules so project-owned code files remain under 200 lines.
- Re-ran Ultracite, TypeScript, React Doctor, rustfmt, Clippy, and the full Cargo workspace tests successfully.

## LAN startup fallback

- Fixed `start_network` failures on hosts where mDNS cannot initialize or advertise. The network service now falls back to bounded UDP discovery instead of refusing to start.
- Setup errors now display the backend's actual returned message, making platform-specific startup failures diagnosable.

## Frontend provider migration

- Removed Biome and the standalone React Doctor CLI.
- Switched Ultracite to Oxlint + Oxfmt and enabled `oxlint-plugin-react-doctor` through `ultracite/oxlint/js-plugins`.
- Added Node 22.18.0 to the mise toolchain because the current Oxc TypeScript config loaders require it.
- Scoped frontend checks to `src` and `vite.config.ts`, preventing long scans of generated Rust and dependency artifacts.

## Phase 5 chat interaction continuation

- Added right-click message actions for copy, reply, edit, delete, and close.
- Added visible @mention token styling with a small pure parser utility.
- Improved message row hierarchy, action controls, context-menu styling, and responsive message presentation.

- Added local message search UI backed by the existing FTS5 query, with room navigation from search results.

- Added direct-message read receipt dispatch when a DM message row is rendered; shared-room messages continue to reject read receipts.

## Phase 5 message transport

- Implemented text `MessagePayload` wire envelopes with Lamport metadata.
- Added per-peer outbound channels and network broadcast for locally persisted text messages.
- Added remote text-message event handling, local replica persistence, and delivered acknowledgements.

- Extended Phase 5 network delivery to edit, delete, and reaction mutation envelopes with remote replica application.

- Completed network delivered/read acknowledgement flow and applied remote edit, delete, reaction, and receipt events to local persistence.

- Added database integration coverage for chat mutations and delivered/read receipt state.

- Added a loopback TLS peer integration test covering handshake completion and live outbound message delivery.

- Added deterministic Lamport-clock ordering coverage and the versioned `SyncRequest` protocol envelope for reconnect synchronization groundwork.

## Phase 5 chat UX fixes

- Corrected feed ordering so oldest messages are above newer messages.
- Removed the persistent inline action button row; message actions now appear through right-click, hover reaction, or touch swipe.
- Constrained the chat shell to one application viewport scroll.
- Added touch swipe actions for reply and reaction.
- Added Enter-to-send and Shift+Enter newline behavior.

- Added message-state Lamport columns and migration-cursor queries for reconnect synchronization.
- Added LWW guards for remote edits and deletes plus a regression test for rejecting an older edit.

## Follow-up Phase 5 UX corrections

- Reworked the message context menu into a viewport-positioned menu that closes on outside click, Escape, and action selection.
- Removed touch swipe actions as requested.
- Added local sender `You` labeling and remote peer username projections.
- Kept hover reaction behavior and fixed message viewport scrolling and composer Enter behavior.

- Added near-bottom auto-scroll for new messages while preserving scroll position during older-history pagination.

- Replaced the message composer textarea with a contenteditable message input and kept Shift+Enter newline support.
- Reply context now clears automatically after a successful send.

## Tauri command split fix

- Fixed generated Tauri command symbol resolution after extracting chat mutations into a submodule.
- Corrected Lamport migration expectations and initialized message state Lamport columns on insert.

- Implemented the first reconnect catch-up response path using Lamport cursors and targeted peer sends.

- Fixed repeated first-launch setup display by restoring the persisted local identity and restarting the LAN service automatically on app reopen.

- Added frontend `zenpaws://event` message reconciliation for incoming messages, remote mutations, and acknowledgement status updates.

- Extended remote message reconciliation to cover delete and reaction events.

## Shadcn/chat UI foundation

- Added Tailwind CSS v4, the official Vite plugin, shadcn `components.json`, shared utilities, and the shadcn Button component.
- Installed Chatcn `message` and `chat-container` components through the documented shadcn registry URL.
- Integrated Chatcn `MessageContent` into the ZenPaws message row and retained the project-owned reaction picker for LAN mutation integration.
- Skipped Chatcn `prompt-input` because its textarea-based API conflicts with the approved contenteditable composer requirement.

- Added Chatcn message presentation and shadcn avatar primitives to live chat rows.

- Completed deleted-message UI projection and placeholder rendering.

## Vite and shadcn path resolution

- Fixed unresolved `@/lib/utils`, `@/components/ui/avatar`, and `@/components/ui/tooltip` imports by adding the Vite `@` alias.
- Added Node typings for the Vite configuration.
- Reduced Vite watcher scope to avoid dependency/toolchain directories.

- Scoped Tailwind v4 source detection to the frontend source tree and restored successful production builds.

- Verified the production Vite build after the Tailwind and Chatcn integration.