# 16 - Project Memory

Always resume from this file rather than re-deriving intent from chat history.

## Phase 0 — Documentation and Architecture (complete)

- ZenPaws is a LAN-only, hostless desktop messenger plus independently
  functioning desktop-pet platform. The app identifier presently configured
  is `com.krishna.zenpaws`; confirm ownership before a public release.
- Replication uses Lamport-clock last-write-wins, tie-broken by peer UUID.
- Transport and discovery decisions were implemented in Phase 2: TCP, `rustls`
  TLS with TOFU pinning, `mdns-sd`, and bounded UDP broadcast fallback.
- The planned SQLite binding is `rusqlite`, with WAL and FTS5, in Phase 3.
- Constraints fixed: SHA-256, a 2 GB file maximum, a 25 MB image
  auto-transfer threshold, and two pet-state broadcasts/second/pet.
- Owner-approved receipt semantics: delivered means acknowledgement from at
  least one directly connected peer; read acknowledgements are DMs only.

## Phase 1 — Project Foundation (complete)

- Rebuilt the foundation around Bun: `packageManager` pins Bun 1.3.14;
  scripts and all contributor instructions use `bun`, never a Node package
  manager runner. `bun.lock` is intentionally absent until the first Bun
  installation can generate it; it must be committed when generated.
- The required Tauri v2 layout is in place: React in `src/`, Tauri package
  and Cargo workspace root in `src-tauri/`, internal `zenpaws-*` crates in
  `src-tauri/crates/`, per-window capabilities, and a thin IPC command.
- Phase-1 Rust dependencies are limited to Tauri, its two necessary plugins,
  and serde for the IPC smoke-test response. Future networking, storage, and
  database dependencies were deliberately not preinstalled.
- The initial shell uses a real `app_status` IPC command to exercise the
  frontend/backend bridge. It is foundation wiring, not a product feature.
- Added a first-party ZenPaws application icon set and restrictive production
  CSP; development CSP permits only the Vite development origin and IPC.
- The project owner confirmed the Phase 1 Clippy remediation and marked this
  phase complete. Phase 2 remains blocked on the delivery and read decision.

## Phase 2 - Networking Foundation (complete)

- Owner approved direct-message-only read acknowledgements. Shared-room
  messages have no read receipt or per-peer seen state.
- Added the first real networking foundation: a typed shared event bus, stable
  peer identity type, explicit connection state machine, and bounded binary
  protocol framing with a protocol version and 1 MiB control-frame limit.
- Added only the dependencies used by this work: `tokio`, `serde`, `uuid`,
  `thiserror`, `bincode`, and `mdns-sd`. The `mdns-sd` 0.18.2 API was checked
  before use: `ServiceDaemon::new`, `browse`, `register`, and async receiver
  events are its documented interfaces. Added `rustls`, `rcgen`, and `sha2`
  for real self-signed TLS identity generation, SHA-256 certificate pins, and
  client configurations that trust only an explicitly stored peer certificate.
- Added `JsonTrustStore` in `zenpaws-settings`. It persists first-use peer
  certificates, accepts reconnects with the same SHA-256 pin, and rejects a
  changed pin instead of silently replacing it. `TlsIdentity::trust_peer_certificate`
  uses that shared trust-store boundary to create a TLS client configuration
  only after the first-use record or pin match succeeds. Added `tokio-rustls`
  and verified a loopback TCP connection is upgraded with the pinned TLS
  configuration. Added a bounded exponential reconnect backoff with reset on
  successful reconnect; it has no polling loop or port scanning behavior.
- Added the post-TLS application handshake. It exchanges and validates UUID
  and username using the framed protocol only after TLS succeeds; a discovery
  UUID mismatch is rejected.
- Added the UDP broadcast fallback with a bounded 1 KiB, versioned discovery
  datagram. It is only a fallback for filtered mDNS networks, never a port
  scanner or polling mechanism.
- Added `NetworkService` to own the TCP listener, mDNS advertisement, TLS
  accept path, post-TLS handshake, and inbound peer lifecycle. It uses the
  configured heartbeat deadline rather than a polling task.
- Added persistent local identity material in `zenpaws-settings`: peer UUID,
  validated username, self-signed certificate DER, and PKCS#8 private key DER.
  `TlsIdentity` can restore those bytes without regenerating a certificate.
- Added `NetworkService::connect_peer` for the outbound lifecycle. It applies
  the stored TOFU certificate decision, opens pinned TLS, runs the application
  handshake, validates the discovered UUID, and creates peer state.
- Added the `start_network` Tauri command. It loads or creates the persisted
  local identity, binds and advertises the service once, and starts the bounded
  inbound lifecycle task. Root Tauri compilation remains blocked in this
  sandbox by the missing Linux `gdk-3.0` development package.
- mDNS advertisements now carry the public certificate as bounded Base64 TXT
  chunks. Resolved peer metadata reconstructs the certificate, endpoint, UUID,
  and username for the first-use TOFU decision and pinned outbound connection.
  `start_network` now owns the JSON trust store and starts the mDNS-driven
  outbound connection task alongside inbound acceptance. Duplicate discovery
  events are deduplicated per peer. Failed outbound connections use bounded
  retry backoff; a TOFU pin mismatch emits `PeerTrustViolation` and does not retry.
- Added `stop_network`, which signals the inbound and discovery tasks through a
  Tokio watch channel and shuts down the mDNS daemon. The runtime remains
  single-start for the current app process to preserve one local identity.
- The UDP fallback now runs in the app lifecycle. It broadcasts the bounded
  advertisement at most once every 30 seconds, receives peer advertisements,
  and routes them through the same deduplicated TOFU and TLS connection flow.

## Phase 3 - Database Foundation (complete)

- `rusqlite` documentation was checked before use. `Connection::open_with_flags` supports explicit read-write/create flags; the `bundled` feature supplies a controlled SQLite build with FTS5 support.
- Added immutable migrations `0001_init.sql`, `0002_messages_fts.sql`, and `0003_peer_certificates.sql`. Existing migration files were never edited.
- Implemented WAL mode, foreign keys, bounded busy timeout, typed peer and settings writes, keyset pagination, FTS5 search, and disk-backed migration coverage.
- Added `DatabaseTrustStore`, which persists certificate DER and SHA-256 pins in SQLite and rejects changed pins. The network runtime now uses it.
- Tauri setup opens the app-data `zenpaws.sqlite` behind an `Arc<Mutex<Database>>`. Root Tauri compilation still requires the unavailable GTK development package in this sandbox.


## Phase 5 - Chat System (in progress)

- Owner approved transition after Phase 4. Start from typed database message APIs, the event bus, and the owner-approved DM-only read receipt policy.- Reconstructed the textual repository files from the Repomix snapshot and restored the local verification environment with mise, Bun 1.3.14, and Rust 1.97.1. Added `.mise.toml` and generated `bun.lock`; all JavaScript package operations now use Bun.
- Fixed the frontend diagnostics required by Ultracite: unstable JSX handler references now use stable callbacks, rejected `void` promise expressions were replaced with handled promises, invalid ARIA markup was replaced with semantic elements, nested status ternaries were simplified, and formatting was normalized.
- Fixed malformed Rust command-module lint attributes, formatted the workspace, added valid RGBA Tauri icon inputs, and installed the Debian Tauri GTK/WebKitGTK development prerequisites. Full workspace Clippy, formatting, compilation, and tests now pass.
- Added React Doctor coverage through the Oxlint plugin after checking its current rules reference. The first scan identified missing reduced-motion handling; `useReducedMotion()` and a CSS `prefers-reduced-motion` fallback now satisfy the rule.
- Added `oxlint-plugin-react-doctor` to the Ultracite Oxlint JS-plugin configuration. React Doctor diagnostics now run inside `bun run check`; there is no separate React Doctor CLI or script. The chat command and network service modules were split along responsibility boundaries so all project-owned Rust, TypeScript, and TSX files remain below the 200-line limit.
- Fixed macOS startup failure when mDNS returned `mDNS operation failed`. mDNS initialization and advertisement are now optional at startup; the documented bounded UDP discovery fallback keeps the LAN service available, while the UI also preserves the backend's actual string error instead of replacing it with a generic message.
- Replaced the direct React Doctor CLI and Biome provider with Ultracite's Oxlint + Oxfmt provider and `oxlint-plugin-react-doctor`. Added Node 22.18.0 to `.mise.toml` for the TypeScript config loaders, removed the standalone React Doctor script, and scoped checks to frontend source plus `vite.config.ts` to avoid scanning generated targets.
- Continued Phase 5 UI work with a native right-click message context menu, stable action callbacks, semantic message-action controls, visual @mention token parsing, and responsive message styling. The interaction remains local until the message-wire replication work is completed.
- Added a local message-search surface backed by the existing FTS5 `useMessageSearch` query. Search results can jump the user to the matching room without adding a second search implementation.
- Added the `mark_message_read` Tauri command and direct-room message-row dispatch. Shared-room messages remain excluded by the database receipt policy.
- Implemented the first live message transport path: connected peer write channels, text `MessagePayload` envelopes, broadcast from `send_message`, remote text persistence through `MessageReceived`, and delivered acknowledgements. Mutation envelopes for edits, deletes, reactions, and read receipts remain.
- Extended the live transport path to edit, delete, and reaction mutations. Local commands now broadcast mutation envelopes, remote sessions publish mutation events, and the Tauri event bridge applies them to the local replica.
- Completed network acknowledgement flow: delivered acknowledgements are applied to the sender's local `message_acks` table, and direct-message read receipts now broadcast `Envelope::Ack::Read` to the remote peer.
- Added database mutation/receipt integration coverage for edit, delete, reaction, delivered, and read state transitions. This is the first automated replica-level verification for the Phase 5 mutation path.
- Added a two-peer loopback TLS integration test that completes both handshakes, starts the live outbound channel, and verifies a message envelope reaches the other peer without mDNS or a second machine.
- Added deterministic Lamport-clock comparison coverage and a `SyncRequest` wire-envelope variant as groundwork for reconnect catch-up. Full sync query/response implementation and LWW mutation application remain pending.
- Fixed current Phase 5 chat UX issues: chronological oldest-to-newest feed order, single application scroll, hidden inline action row with context-menu-only actions, hover reaction, touch swipe actions, and Enter-to-send with Shift+Enter for new lines.
- Added migration `0004_message_lww`, Lamport-cursor reconnect queries, idempotent replicated inserts, and guarded remote edit/delete application so older Lamport mutations cannot overwrite newer visible state.
- Fixed the message context menu to use viewport coordinates, close on outside click or Escape, and close after actions. Removed swipe actions entirely. Sender rows now display `You` from the persisted local peer ID, remote rows use persisted peer usernames, and the feed no longer creates an application-level second scrollbar.
- Added near-bottom auto-scroll for initial and incoming messages while preserving the user's position when they are reading older history. Loading older pages no longer forces a jump to the bottom.
- Replaced the composer textarea with an accessible contenteditable message input while preserving Enter-to-send and Shift+Enter newlines. Reply context now clears automatically after a successful send.
- Fixed the Tauri command-module split regression: mutation commands now live in a public `commands::chat::mutations` module, retain their generated Tauri command symbols, and import the network and status types from their correct module paths. The database migration test expectations were also corrected to account for `0004_message_lww`.
- Added reconnect sync request handling: a new peer sends `SyncRequest`, the Tauri event bridge queries `messages_after_lamport`, and the network service sends matching text envelopes back to the requesting peer.
- Fixed repeated setup-screen display: when both the persisted username and peer ID exist, AppShell automatically restarts the LAN service and enters chat; setup is shown only for a missing or failed local identity.
- Added a frontend event bridge hook that invalidates message and receipt queries when remote messages, mutations, or acknowledgements arrive, so the chat feed and status indicators reconcile without manual reload.
- Extended the frontend event bridge to invalidate the feed for remote deletes and reactions as well as remote edits and incoming messages.
- Added Tailwind CSS v4 and the shadcn/ui Vite foundation, then installed the Chatcn message and chat-container components from the documented registry URL without the stale `.json` suffix. The Chatcn prompt-input component was intentionally not used because the ZenPaws composer uses a contenteditable input.
- Integrated Chatcn MessageContent and shadcn Avatar/AvatarFallback into the live message row while preserving ZenPaws actions, author identity, and LAN callbacks.
- Completed delete-state projection: deleted messages now expose a deleted flag to the frontend and render as `Message deleted` instead of showing the old body.
- Fixed Vite dependency-scan resolution by adding the `@` alias to `vite.config.ts`, matching TypeScript and shadcn imports. Added `@types/node` for the Vite config and ignored Bun, Cargo, Rustup, local-tool, node_modules, and Tauri generated directories to prevent unnecessary file watchers.
- Fixed the Tailwind v4 source scan scope with `@import "tailwindcss" source("../")`, which stopped Vite production builds from scanning the entire workspace/toolchain and resolved the previous SIGKILL build failure.
- Production frontend build now passes after Tailwind source scoping and the Chatcn/shadcn integration. The generated bundle is approximately 433 KB JavaScript and 40 KB CSS before gzip.
- Reconciled the manually supplied root Oxc files. Removed the stale duplicate `.oxlintrc.json` that prevented config loading, preserved the manually selected Ultracite Oxlint core plus React presets, removed stale Biome editor actions, ran `bun run fix`, and manually resolved the remaining 17 Oxlint diagnostics. `bun run check` and `bun run typecheck` now pass.
- Hardened the virtualized message feed against malformed or temporarily null infinite-query page data. It now filters non-array pages before flattening, preventing runtime spread/iterability crashes during startup or failed IPC responses.
- Added a dedicated full-screen Network Diagnostics view with local LAN address, mDNS/fallback state, TCP/UDP bindings, connected-peer count, backend event log, and Wi-Fi/firewall/client-isolation checklist. The network service now retains recent startup logs and reports mDNS startup/advertisement failures instead of silently swallowing them.
- Fixed Tauri diagnostics serialization by applying camelCase serde names to `NetworkDiagnostics` and `NetworkStatus`, matching the TypeScript IPC contracts. The previous diagnostics screen showed fallback values because Rust returned snake_case field names.
- Added detailed mDNS discovery/connectivity logs for discovered peers, duplicate/self advertisements, connection failures, TOFU mismatches, and successful connections. Diagnostics now exposes these recent backend events on a separate screen.
- Diagnosed macOS `HostUnreachable` UDP broadcast failures and replaced global `255.255.255.255` fallback targeting with a directed `/24` LAN broadcast derived from the local address, such as `192.168.1.255`. Added UDP loop-start, announcement, receive, and connection-attempt logs for both peers.
