# 02 - Product Requirements Document

Each requirement below is written so it's testable. "Support X" from the original brief is expanded into what "supported" means to verify.

## Identity & presence

- A user sets a username and avatar on first launch; a UUID is generated and persisted locally as their durable identity.
- Peers on the LAN appear in a presence list within a few seconds of joining (mDNS discovery latency, not polling).
- Presence states: online, away, offline - driven by the connection-state machine in `06_NETWORK_ARCHITECTURE.md`, not by manual toggling only.

## Messaging

- Shared room + 1:1 direct messages.
- Text messages support: edit, delete, reply-threading, emoji reactions, @mentions, typing indicators, sent/delivered/read status (semantics defined in `07_MESSAGE_PROTOCOL.md`), full-text search (FTS5), copy, right-click context menu.
- A room with 100,000+ messages scrolls smoothly (virtualized list, paginated fetch) on the target hardware in `13_PERFORMANCE_GUIDELINES.md`.

## Images

- Paste (Ctrl+V), clipboard, and drag-and-drop (browser/desktop/file explorer) all produce an attached image.
- Multiple images per message; preview, zoom, fullscreen, save-to-disk.
- Images auto-transfer to peers (thumbnail first, lazy-loaded original) - the only file type that auto-transfers content, not just metadata.

## Files

- Any file type is attachable. Non-image files transfer **metadata only** on send; content transfers only when the receiver clicks download.
- Transfer supports pause/resume/cancel, progress, and integrity verification (SHA-256) before a downloaded file is considered complete.

## Desktop pets

- Seven initial pets (cat, dog, fox, rabbit, slime, ghost, parrot), each a transparent always-on-top window, data-driven from `assets/pets/<pet>/manifest.json` - installable without a code change.
- Pets react to chat activity (typing, new message, celebration) and to cursor behavior (chase, react to rapid movement).
- Click-through mode is toggleable per pet.

## Notifications

- Native OS notification + pet speech-bubble on new message/mention.
- Clicking a notification focuses the relevant chat.
- No message content in the notification preview by default.

## Settings

- Username, avatar, theme (light/dark/system), notification prefs, download location, auto-download, startup behavior, font size, pet settings, performance mode (Full vs. Performance).

## System integration

- System tray with background operation; window state persists across restarts; clipboard copy/paste for text and images.

## Non-functional requirements

See `13_PERFORMANCE_GUIDELINES.md` (performance budgets) and `12_SECURITY_MODEL.md` (threat model) - both are requirements, not aspirations, and Phase 10 testing verifies against them explicitly.