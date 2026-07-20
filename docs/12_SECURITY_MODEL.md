# 12 - Security Model

## Threat model - explicit boundary

**Protects against**: peer identity spoofing on the LAN, casual traffic
snooping by other devices on the same subnet, path traversal / malicious
filenames from a peer, oversized/malformed transfer abuse.

**Does not protect against** (see `03_PROJECT_SCOPE.md` non-goals): a
sophisticated attacker who already has a privileged foothold on the LAN or
on a peer's machine itself; this is a trusted-LAN tool, not a
zero-trust-network product.

## Transport security

TLS via `rustls`, with **TOFU (Trust On First Use)** peer-identity pinning:

1. On first contact with a peer UUID, the connecting peer records that
   peer's certificate/public-key fingerprint (`peers.pubkey_fingerprint` in
   `09_DATABASE_SCHEMA.md`).
2. On every subsequent connection, the presented fingerprint must match the
   pinned one - a mismatch is surfaced to the user as a warning (possible
   impersonation or a peer that reinstalled/regenerated identity), not
   silently accepted or silently rejected.
3. This is how "UUID isn't IP" identity claims get verified - a UUID+
   username alone is just an assertion; the pinned key is what makes it
   costly to spoof after first contact.

## File handling

- **Filename sanitization**: `originalFilename` is display-only, never used
  as a filesystem path; `storedFilename` is always a UUID assigned by the
  receiver.
- **Path traversal protection**: all file writes are canonicalized and
  checked to stay within the configured storage root before any write
  occurs.
- **Checksum validation**: SHA-256, verified before a transferred file is
  considered complete (`08_FILE_TRANSFER_PROTOCOL.md`).
- **Upload limits**: 2 GB per file, enforced before transfer begins.

## Tauri v2 capability scoping (per window)

Tauri v2's per-window capability files (`src-tauri/capabilities/*.json`)
replace v1's single global allowlist - this is used for real least-privilege
separation:

- **`main-window`**: the full legitimate command surface - filesystem
  access scoped to the app's own data/downloads directories, dialog,
  clipboard, notification, tray.
- **`pet-*` windows**: minimal - window positioning/transparency/click-
  through and the specific pet-state IPC commands. No filesystem, network,
  or dialog access, since pet windows render frequently-updated asset
  content and don't need it.

The exact permission-string list in each capability file is finalized
against Tauri's generated schema (`src-tauri/gen/schemas/`, produced on
first `cargo tauri dev` once plugins are registered) rather than guessed -
the Phase 1 capability files ship a first-pass list to be verified then.

Whenever a new `#[tauri::command]` is added in any phase, its reachability
from a given window is re-checked against that window's capability file -
a command existing in Rust does not make it callable; the capability grant
does.

## Database access

All SQL is parameterized/prepared statements through `zenpaws-database`'s
typed functions. The frontend never sends raw SQL - this was also a factor
in choosing `rusqlite` over `tauri-plugin-sql` in `05_TECHNICAL_DECISIONS.md`.

## Local data at rest

Not encrypted at rest in this build (LAN-trust threat model, see above). If
that changes, it's a new decision recorded here, not an assumption.
