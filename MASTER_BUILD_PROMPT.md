# MASTER_BUILD_PROMPT.md - ZenPaws

## Role

You are a senior software architect, senior Rust engineer, senior Tauri engineer,
senior React engineer, networking engineer, distributed systems engineer, desktop
application engineer, security engineer, and performance optimization specialist.

Your task is to build **ZenPaws**, a production-grade, cross-platform, LAN-only
peer-to-peer desktop messenger with interactive desktop pets, built on Tauri v2.

App identifier convention: `com.zenpaws.app` (used in `tauri.conf.json` and
platform bundle identifiers - confirm/replace with the real reverse-domain
identifier before Phase 1).

Technology stack:

- Tauri v2
- Rust
- Tokio
- React 19
- TypeScript
- Vite
- Zustand
- TanStack Query
- SQLite
- SQLite FTS5
- SQLite WAL mode

---

# Core Philosophy - `DietrichGebert/ponytail` Ruleset

This project follows the ponytail engineering discipline. It is a coding
discipline, not an architectural or pet-system reference - ponytail has no
rendering, window, or pet code. Before implementing anything, always evaluate
in order:

1. Does this feature need to exist?
2. Can an existing module solve it?
3. Can the Rust/TypeScript standard library solve it?
4. Can the OS or Tauri solve it natively?
5. Can an already-installed dependency solve it?
6. Can it be implemented with significantly less code?
7. Only then write new code - the minimum that correctly solves the problem.

Never introduce abstractions, services, dependencies, patterns, or layers
without a clear, stated need. Record any deliberate override of this ladder,
with justification, in `05_TECHNICAL_DECISIONS.md`.

**Non-negotiable exception:** do not sacrifice security, correctness,
reliability, or accessibility for simplicity. Laziness applies to avoiding
unnecessary abstraction - never to cutting a safety or correctness concern.

If desktop-pet visual/architectural inspiration is wanted from a *different*
named project (rendering, transparent overlay windows, animation, click-
through), name it explicitly with its GitHub URL before it's used as a
reference - that is unrelated to the ponytail ruleset above. Study technique
only; never copy code or art assets verbatim, since they may be licensed.

---

# Project Vision

Three independent systems sharing a common platform:

1. **Messenger** - LAN-based chat.
2. **Desktop Pet Platform** - interactive desktop pets.
3. **LAN Networking Platform** - peer discovery, synchronization, messaging,
   file transfer.

These systems remain loosely coupled via the Event Bus (see below). The pet
engine must remain functional even if chat functionality is disabled.

---

# Non-Goals

Explicit scope boundaries, to prevent drift during implementation:

- No internet/WAN relay, NAT traversal, or cloud component of any kind.
- No mobile clients.
- No multi-workspace / multi-server topology - a single flat LAN room plus
  DMs, not a Slack-style architecture.
- No encryption/identity guarantees beyond what's specified in the Security
  section - this is a LAN tool, not designed to resist a nation-state
  adversary on the same subnet, but it must resist casual snooping and
  spoofing by other devices on that LAN.

---

# Supported Platforms

Windows, Linux, macOS. Architecture must remain cross-platform. Avoid
platform-specific code unless unavoidable. When it's required: isolate it,
document it, and provide fallback behavior.

---

# Development Rules

- Never generate the entire project at once. Work phase-by-phase.
- Wait for explicit approval before starting the next phase.
- Never silently change architecture decisions.
- Never generate placeholder implementations unless explicitly requested.
  Avoid TODO-only code, fake implementations, mocked production features, or
  temporary architecture.
- If a requirement is ambiguous or two requirements conflict, stop and ask
  rather than silently choosing an interpretation.
- After every phase: update documentation, memory, backlog, and changelog,
  and commit that phase as one atomic git commit with a descriptive message
  (`phase-N: <summary>`) so any phase can be reverted independently.

---

# Development Phases

| Phase | Name |
|---|---|
| 0 | Documentation and Architecture |
| 1 | Project Foundation |
| 2 | Networking Foundation |
| 3 | Database Foundation |
| 4 | Pet Engine |
| 5 | Chat System |
| 6 | Image System |
| 7 | File Transfer System |
| 8 | Notification System |
| 9 | Performance Optimization |
| 10 | Testing and Hardening |

Never skip phases.

---

# Documentation Structure

```
docs/
00_PROJECT_CONSTITUTION.md
01_PROJECT_OVERVIEW.md
02_PRODUCT_REQUIREMENTS_DOCUMENT.md
03_PROJECT_SCOPE.md
04_SYSTEM_ARCHITECTURE.md
05_TECHNICAL_DECISIONS.md
06_NETWORK_ARCHITECTURE.md
07_MESSAGE_PROTOCOL.md
08_FILE_TRANSFER_PROTOCOL.md
09_DATABASE_SCHEMA.md
10_PET_ENGINE_SPECIFICATION.md
11_UI_UX_GUIDELINES.md
12_SECURITY_MODEL.md
13_PERFORMANCE_GUIDELINES.md
14_DEVELOPMENT_PHASES.md
15_AI_AGENT_RULES.md
16_PROJECT_MEMORY.md
17_CHANGELOG.md
18_BACKLOG.md
19_KNOWN_ISSUES.md
20_TESTING_STRATEGY.md
21_DEPLOYMENT_GUIDE.md
22_CONTRIBUTING_GUIDE.md
23_ASSET_LICENSES.md
24_API_REFERENCE.md
25_EVENT_BUS_ARCHITECTURE.md
26_STORAGE_STRATEGY.md
27_SYNC_REPLICATION_MODEL.md
```

`05_TECHNICAL_DECISIONS.md` must record concrete choices, not categories -
which mDNS crate, which SQLite binding, which hashing algorithm, which
conflict-resolution strategy - so later phases don't re-litigate them.

---

# Project Structure

This must follow Tauri v2's required layout - Tauri isn't flexible about this
part: the CLI locates the Rust project by finding `tauri.conf.json` next to
`Cargo.toml` inside `src-tauri/`, and expects `src-tauri/src/main.rs` +
`lib.rs`, `capabilities/`, and `icons/` at fixed locations. Modularity
(network/database/pets/etc.) is achieved with a **Cargo workspace inside
`src-tauri/`**, not with invented top-level folders Tauri doesn't recognize.

```
zenpaws/
├── package.json
├── vite.config.ts
├── index.html
├── src/                          # React frontend (Vite root)
│   ├── main.tsx
│   ├── app/                      # routing/shell
│   ├── features/                 # chat/, pets/, transfers/, settings/
│   ├── stores/                   # Zustand
│   ├── queries/                  # TanStack Query
│   ├── components/
│   └── shared/
│
└── src-tauri/                    # Rust project - Tauri-required layout
    ├── Cargo.toml                # workspace root
    ├── Cargo.lock
    ├── build.rs                  # tauri_build::build()
    ├── tauri.conf.json           # app id, windows, bundle config
    ├── icons/
    ├── capabilities/             # per-window permission scoping (see Security)
    │   ├── main-window.json
    │   └── pet-window.json
    ├── src/
    │   ├── main.rs                # entrypoint only - do not modify; calls app_lib::run()
    │   ├── lib.rs                 # tauri::Builder setup, plugin registration
    │   └── commands/              # thin #[tauri::command] handlers only -
    │                               # delegate to the crates below, no business logic here
    └── crates/                   # internal workspace crates - the actual modularity
        ├── zenpaws-shared/        # common types, error handling, the Event Bus
        ├── zenpaws-network/       # discovery, transport, peer/session mgmt
        ├── zenpaws-database/      # schema, migrations, queries (SQLite/WAL/FTS5)
        ├── zenpaws-pets/          # pet state machine, asset loading, window mgmt
        ├── zenpaws-transfer/      # chunked file transfer service
        ├── zenpaws-notifications/ # native + in-app notification dispatch
        ├── zenpaws-storage/       # on-disk file storage, quotas, cache
        └── zenpaws-settings/      # user preferences, persistence
```

Each `zenpaws-*` crate is a normal library crate with its own `Cargo.toml`,
added as a path dependency of the `src-tauri` binary crate. `commands/`
handlers stay thin - they translate a Tauri IPC call into a call on the
relevant crate and back, so business logic is unit-testable without spinning
up a Tauri runtime. Cross-crate communication goes through the Event Bus
defined in `zenpaws-shared`, not direct crate-to-crate coupling, preserving
the loose-coupling requirement from the Project Vision section.


---

# Networking

LAN only. No cloud, no internet relay, no external servers.

- **Discovery:** mDNS/Bonjour primary, UDP broadcast fallback.
- **Identity:** UUID + username + avatar. IP is a transport detail only,
  never identity (DHCP can reassign it).
- **Transport:** TCP with a binary protocol / binary streaming. Avoid
  WebSocket unless Phase 0 documents a specific, strong justification for it.
- **Resilience:** auto-reconnect, heartbeat, retry with backoff, explicit
  connection-state machine (connecting / connected / degraded / disconnected).
- Avoid unnecessary polling and port scanning. Minimize LAN traffic.
- **Protocol versioning:** every message envelope includes a protocol
  version field, independent of the replication message-versioning below, so
  future protocol changes don't silently break peers on an older build.
- **Delivery semantics:** given the hostless replication model (no central
  server), define explicitly in `07_MESSAGE_PROTOCOL.md` what "sent,"
  "delivered," and "read" mean - e.g. delivered = acknowledged by at least
  one directly-connected peer, read = acknowledged by the specific
  recipient. Don't leave this implicit; a P2P mesh has no default answer.

---

# Replication Model

Hostless replication. There is no server. Every peer stores shared-room
history, metadata, and local settings. Peers synchronize missing messages
after reconnecting.

Implement and document in `27_SYNC_REPLICATION_MODEL.md`:

- Message versioning and the synchronization protocol.
- **A named conflict-resolution mechanism** - e.g. last-write-wins with
  logical/Lamport clocks, or a CRDT - not just "conflict handling" as a
  category. State which one and why before Phase 3 implementation begins.

---

# Event Bus Architecture

All major modules communicate through an event bus rather than direct
coupling - e.g. `Network → EventBus → Chat`, `Network → EventBus → Pets`,
`Network → EventBus → Notifications`, `Network → EventBus → Transfers`.
Document the event taxonomy in `25_EVENT_BUS_ARCHITECTURE.md`.

---

# Messaging Features

Shared room + direct messages. Support: text, timestamps, edit, delete,
reply, emoji reactions, mentions, typing indicators, message status, search,
copy, context menus.

Must support 100,000+ messages via pagination, virtualization, and indexing.

---

# Database

SQLite with WAL mode, FTS5, prepared statements, migrations.

Never edit an existing migration - always create a new one.

---

# Image Features

Ctrl+V paste, clipboard image, drag from browser/desktop/file explorer,
multiple upload, preview, zoom, fullscreen, save, lazy loading. Images may
auto-transfer. Generate thumbnail + metadata via a dedicated
`ThumbnailService`; display thumbnail first, load originals lazily.

---

# File Features

Supported: PDF, ZIP, DOCX, TXT, CSV, MP4, images, arbitrary files.

Upload flow: generate UUID filename, store metadata in SQLite (`fileId`,
`messageId`, `senderId`, `mimeType`, `size`, `checksum`, `createdAt`,
`originalFilename`, `storedFilename`).

- **Checksum algorithm: SHA-256**, used consistently for both file-transfer
  integrity checks and any content-addressed storage.
- **Upload size limit:** document a concrete maximum (e.g. a specific MB
  figure appropriate to LAN bandwidth) in `05_TECHNICAL_DECISIONS.md` -
  "upload limits" must resolve to an actual number before Phase 7, not stay
  a placeholder.

Non-image files: never auto-transfer content, only metadata. Receiver sees a
download button; on click, requests the file, sender streams it, receiver
saves it to the downloads folder or a configured location.

---

# File Transfer

Dedicated `TransferService`, never coupled to chat logic. Chunked transfer
with pause/resume/cancel, progress reporting, and SHA-256 checksum/integrity
validation before marking complete.

---

# Desktop Pet System

First-class feature. Transparent, always-on-top windows with click-through
mode. Data-driven - no hardcoded pets; installable without code changes.

Initial pets: cat, dog, fox, rabbit, slime, ghost, parrot.

```
assets/pets/<pet>/
  manifest.json
  sprites/
  sounds/
  LICENSE.txt
```

**States:** idle, walk, run, jump, sleep, typing, notification, celebrate,
fight, play, away, offline.

**Behaviors:** chase cursor, react to rapid cursor movement, interact with
other pets, celebrate on messages, display notifications, idle behaviors.

**Future (not this build):** one pet per LAN user.

---

# Pet Synchronization

State synchronization only. Never synchronize coordinates, frame positions,
or per-frame animation state. Synchronize discrete events only: typing,
playing, sleeping, celebrating, notification, away, offline.

**Rate limit: maximum 2 pet-state broadcasts per second per pet**, with
debounce, deduplication, and rate limiting implemented explicitly - this
matters at the target scale of up to 50 concurrent peers.

---

# Notifications

Native OS notifications + pet speech-bubble notifications. Clicking a
notification opens the relevant chat. Privacy-first: no message content in
notification previews by default.

---

# Settings

Username, avatar, theme (light/dark/system), notifications, download
location, auto-download, startup behavior, font size, pet settings,
performance mode.

---

# System Features

System tray, background operation, auto-start, window-state persistence.
Clipboard: copy/paste text and images.

---

# Security

- Filename sanitization, path traversal protection, UUID-based file
  identifiers, SHA-256 checksum validation, upload size limits (see File
  Features).
- **Transport security:** TLS with TOFU (Trust On First Use) peer-identity
  pinning - since IP isn't identity, this is how a UUID+username claim gets
  validated on first contact and re-verified on reconnect.
- Document the threat model and identity-verification flow explicitly in
  `12_SECURITY_MODEL.md`: what this protects against (spoofing, casual
  snooping by other LAN devices) and what it explicitly does not (see
  Non-Goals).

## Tauri v2 Capability Scoping (per window)

Tauri v2 replaces v1's single global allowlist with per-window capability
files in `src-tauri/capabilities/`. This must be used for real least-
privilege separation, not left at defaults:

- **Main window** (`main-window.json`): the full command surface it
  legitimately needs - filesystem access scoped to the app's data/downloads
  directories, dialog, clipboard, notification, tray.
- **Pet windows** (`pet-window.json`): a deliberately minimal capability
  set - window positioning/transparency/click-through and the specific IPC
  commands pets need (state updates, asset loading). No filesystem,
  network, or dialog access from a pet window's webview context, since pet
  windows render less-trusted, frequently-updated asset content.

Document the exact permission list per window (not just "least privilege" as
a principle) in `12_SECURITY_MODEL.md`, and revisit it whenever a new
`#[tauri::command]` is added - a command is only reachable from a window if
that window's capability file explicitly grants it.

---

# Storage Strategy

Implement quotas and document limits in `26_STORAGE_STRATEGY.md` for: image
cache, thumbnail cache, temporary in-progress transfers, and logs. Prevent
uncontrolled storage growth on long-running installs.

---

# Performance Mode

Two runtime modes:

- **Full Mode** - maximum animations and interactions.
- **Performance Mode** - reduced animation rate, reduced effects, lower pet
  FPS, reduced resource usage.

---

# Performance Targets

Target hardware: dual-core Intel i3-equivalent CPU, 4GB RAM, HDD supported,
integrated GPU.

| Metric | Target |
|---|---|
| Idle RAM | < 200 MB |
| Idle CPU | < 2% |
| Pet engine | < 1% CPU per active pet |
| 100,000-message history | Smooth scrolling |
| Large image history | Lazy loaded |

---

# Testing

Write `20_TESTING_STRATEGY.md` **before Phase 1 begins**, not retrofitted
per phase - define the unit, integration, replication, LAN multi-device, and
performance testing approach up front so later phases build to it rather
than around it. No phase is complete without its testing documentation
updated accordingly.

---

# AI Memory Rules

After every phase, update `16_PROJECT_MEMORY.md`, `17_CHANGELOG.md`,
`18_BACKLOG.md`, `19_KNOWN_ISSUES.md` with: completed work, pending work,
architecture decisions, tradeoffs, risks. Always continue from memory files;
never rely solely on prompt context.

---

# Output Rules

Start with **Phase 0 only**. Phase 0 must include: documentation structure,
folder structure (the actual Tauri v2 layout - `src/` + `src-tauri/` with its
internal Cargo workspace), architecture diagrams, technology decisions,
networking decisions, replication decisions (including the named conflict-
resolution mechanism), event bus design, security decisions (including the
transport security / TOFU flow and the per-window capability files for
main-window vs. pet-window), and database decisions.

Do not generate implementation code until Phase 0 is reviewed and approved.

At the end of every phase, provide:

- Definition of Done
- A brief self-review against the ponytail ladder (what was skipped as
  unnecessary, what reused an existing dependency, what needed genuinely new
  code and why)
- Open Questions
- Risks
- Next Phase Plan

Wait for approval before proceeding.
