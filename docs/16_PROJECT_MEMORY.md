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

- Owner approved transition after Phase 4. Start from typed database message APIs, the event bus, and the owner-approved DM-only read receipt policy.

