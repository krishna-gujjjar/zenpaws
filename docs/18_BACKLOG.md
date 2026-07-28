# 18 - Backlog

## Deferred to later phases

- Phases 6/7: dialog and clipboard plugins, storage and transfer logic.
- Phase 8: notification and autostart plugins.
- Phase 5: detailed visual design tokens against real screens.
- Review generated pet sprite alpha, frame boundaries, and commercial-use provenance before committing production asset packages.

## Non-goals

WAN relay, mobile clients, multi-workspace topology, and one pet per LAN user are outside this build.

## Phase 5 remaining work

- Add multi-peer integration coverage for text, edit, delete, reaction, and acknowledgement delivery; the live transport and local replica paths are now implemented.
- Persist extracted mention identities and validate them against discovered peer usernames before sending mention events.
- Add network delivery of direct-message read acknowledgements. The room-level search surface is now implemented using the existing local FTS5 query.
- Add component tests for the composer, context menu, mention parser, and startup error states before closing the phase.

- Apply LWW mutation conflicts to every streamed reconnect mutation and add multi-peer reconnect integration coverage. Text catch-up request/query/response is now implemented.