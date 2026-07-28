# 20 - Testing Strategy

Written before Phase 1 implementation begins, per `00_PROJECT_CONSTITUTION.md`'s
process rules - later phases build to this, not around it.

## Layers

| Layer | Tool | Scope |
|---|---|---|
| Rust unit tests | `cargo test` / `cargo nextest run` | Pure logic inside each `zenpaws-*` crate - Lamport clock ordering, checksum verification, manifest parsing, etc. |
| Rust integration tests | `cargo nextest run`, loopback TCP between two in-process peers | Handshake, TOFU pinning, message round-trip, sync-on-reconnect - without real hardware. |
| Frontend unit/component tests | Vitest + React Testing Library | Component behavior, hooks, pure utils - no `any` in test code either. |
| Replication tests | Rust integration tests with 3+ simulated peers on loopback | Conflict resolution (concurrent edits), late-join catch-up sync. |
| LAN multi-device tests | Manual checklist (below) | Real mDNS discovery, real NAT-free LAN conditions, cross-OS pairing (Windows↔Linux↔macOS). |
| Performance tests | `criterion` (Rust benches), manual profiling on target-spec hardware | Verifies the budgets in `13_PERFORMANCE_GUIDELINES.md`. |

## CI gates (from Phase 1 onward)

Every commit: `bun run check` (Ultracite with Oxlint, Oxfmt, and the React Doctor Oxlint plugin), `cargo clippy --all-targets -- -D warnings`,
`cargo fmt --check`, `tsc -b --noEmit`, `cargo nextest run`, `vitest run`.
A phase is not done if any of these fail.

## Manual LAN multi-device checklist (used starting Phase 2, finalized Phase 10)

- [ ] Two peers on the same LAN discover each other within a few seconds.
- [ ] A peer joining after messages exist catches up via sync, per
      `27_SYNC_REPLICATION_MODEL.md`.
- [ ] TOFU mismatch (simulated by regenerating a peer's key) surfaces a
      warning, not a silent failure.
- [ ] A large file transfer (near the 2GB limit) survives a pause/resume
      cycle and passes checksum verification.
- [ ] Idle CPU/RAM on target-spec hardware stays within the budgets in
      `13_PERFORMANCE_GUIDELINES.md` with 5+ peers connected.
- [ ] Cross-OS pairing: Windows↔Linux, Windows↔macOS, Linux↔macOS all work.

## What's explicitly not covered

Load testing at the full 50-peer target is a Phase 9/10 concern once
there's a real build to test - not simulated abstractly here.

## Phase 2 coverage

- Unit tests cover framed envelopes, retry caps, connection degradation, TOFU
  pin changes, local identity persistence, UDP loopback discovery, pinned TLS
  loopback, post-TLS handshakes, and network-service task sharing.
- Manual LAN verification remains required for mDNS and UDP broadcast across
  actual devices because multicast and broadcast behavior is OS and network
  dependent.

## Phase 3 coverage

- Database tests cover migrations, WAL mode, migration reuse on disk, typed writes, FTS5 search, settings persistence, and database-backed TOFU pin rejection.

## Phase 4 coverage

- Pet tests cover local state deduplication, rate limiting, and local-only versus synchronized state mapping.
- Desktop-host testing must verify transparent and click-through pet overlays on Windows, Linux, and macOS.
## Verification completed after environment restoration

The documented gates were executed after installing the pinned mise, Bun, and
Rust toolchains and the Linux Tauri development libraries:

```bash
bun run verify
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

All four commands passed. The remediation included handling the Ultracite
callback, promise, accessibility, and formatting diagnostics; correcting the
malformed Rust command-module attributes; and restoring the RGBA Tauri icons
needed for root-crate compilation. These checks should be rerun after every
phase change, not only before a release.

## React Doctor through Oxlint

React Doctor is integrated through `oxlint-plugin-react-doctor` in
`oxlint.config.ts`. It is intentionally not installed or run as a separate
CLI. The single command below covers Oxfmt, Oxlint, and the React Doctor plugin:

```bash
bun run check
```

The gate must report no issues. In particular, every Motion animation must
respect reduced-motion preferences through `useReducedMotion()` or an
appropriate CSS fallback.

## Single-computer Phase 5 testing

A single running ZenPaws window can verify the local-first chat surface:

```bash
mise install
bun install --frozen-lockfile
bun run tauri dev
```

Enter a username, then verify sending, editing, deleting, replying, reactions,
copying, the right-click context menu, @mention rendering, local FTS search,
and DM read-receipt dispatch where a DM room is available. Restart the app to
verify local message persistence.

Cross-peer delivery cannot be validated by one window yet. The current Phase 5
wire work defines and tests the `MessagePayload` envelope, but the live network
broadcast/receive registry is still pending. Once that is implemented, a
single computer can use two isolated app-data profiles and separate listeners,
or the LAN checklist can use a second machine.

## Phase 5 mutation coverage

The database integration test now applies an edit, reaction, delivered receipt,
read receipt, and delete to one replica and verifies both receipt flags. The
network suite covers message and acknowledgement envelope round trips.
A two-peer loopback service test remains needed to verify the complete transport
path rather than only its protocol and replica layers.

## Loopback peer transport coverage

The network crate now includes a loopback TLS peer test that creates a server
and client session, completes both application handshakes, starts the peer
outbound channel, and verifies a message envelope arrives on the other side.
This validates the live framed transport path without mDNS or a second device.

## Lamport and reconnect coverage

Database coverage now includes idempotent replicated inserts, Lamport-cursor
message queries, and a regression test proving an older remote edit cannot
replace a newer visible edit. The sync request envelope and cursor query are
ready for the reconnect response implementation.
