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

Every commit: `biome check .`, `cargo clippy --all-targets -- -D warnings`,
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
