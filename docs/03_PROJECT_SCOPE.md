# 03 - Project Scope

## In scope (this build)

- LAN-only chat (shared room + DMs), images, arbitrary file transfer.
- Seven data-driven desktop pets with discrete-event state sync.
- Hostless replication among peers (no server).
- Windows/Linux/macOS via Tauri v2.
- TLS + TOFU transport security on the LAN.

## Explicitly out of scope (non-goals)

- **No WAN/internet relay, NAT traversal, or cloud component of any kind.** If a future need for cross-network chat emerges, it's a new project decision, not an extension bolted onto this one.
- **No mobile clients.** Desktop only.
- **No multi-workspace / multi-server topology.** One flat LAN room plus DMs - not a Slack-style architecture with channels-of-channels.
- **No resistance to a nation-state-level adversary already on the LAN.** The security model (TLS + TOFU) protects against casual snooping and identity spoofing by other LAN devices, not a sophisticated attacker who already has a foothold on the same subnet. This line is deliberate - see `12_SECURITY_MODEL.md` for the full threat model.

## Deferred to a later iteration (not this build, but designed for)

- **One pet per LAN user**, broadcast via the same discrete-event sync already built for local pets - the sync protocol in `09_PET_ENGINE_SPECIFICATION.md` is designed so this is additive, not a rework.

## MVP vs. full build

Phases 0–8 (through Notification System) constitute a usable MVP. Phases 9 (Optimization) and 10 (Testing & Hardening) are required before this is "production-grade" in the sense the constitution uses that word - they are not optional polish.