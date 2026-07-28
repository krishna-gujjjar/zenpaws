# 13 - Performance Guidelines

## Target hardware

Dual-core Intel i3-equivalent CPU, 4GB RAM, HDD supported, integrated GPU.

## Budgets

| Metric                  | Target                                  |
| ----------------------- | --------------------------------------- |
| Idle RAM                | < 200 MB                                |
| Idle CPU                | < 2%                                    |
| Pet engine              | < 1% CPU per active pet                 |
| 100,000-message history | Smooth scrolling (virtualized, no jank) |
| Large image history     | Lazy loaded, never all-at-once          |

These are requirements verified in Phase 10, not aspirations - Phase 9 exists specifically to close any gap between where Phases 1–8 land and these numbers.

## How each requirement is met

- **Idle RAM/CPU**: no polling loops; mDNS and heartbeat are event/timer driven with conservative intervals tuned for the 50-peer LAN scale (`06_NETWORK_ARCHITECTURE.md`); the frontend doesn't re-render on data it doesn't display (see Zustand/TanStack Query selector discipline below).
- **Pet engine CPU**: sprite-sheet animation at a capped FPS (data-driven per pet, `10_PET_ENGINE_SPECIFICATION.md`), Performance Mode reduces this further; pet windows are otherwise idle (no continuous redraw when a pet is in `idle` state beyond its own low-fps animation).
- **100k-message scrolling**: `@tanstack/react-virtual` renders only visible rows; SQLite pagination is keyset-based (`created_at` cursor, not `OFFSET`) per `09_DATABASE_SCHEMA.md`; FTS5 search is indexed, not a table scan.
- **Lazy image loading**: thumbnail-first, original fetched only when the user actually views it full-size (`08_FILE_TRANSFER_PROTOCOL.md`'s auto-transfer threshold already limits what arrives unprompted).

## Frontend render discipline

- Zustand selectors are narrow - a component subscribes to the smallest slice of state it needs, not the whole store, to avoid unnecessary re-renders on unrelated state changes.
- TanStack Query cache keys are structured so a chat-list update doesn't invalidate unrelated queries (e.g. settings, pet state).
- Motion animations respect `useReducedMotion` and are scoped to elements actually changing - no blanket page-level animation wrappers.

## Performance Mode

A user-facing toggle (`02_PRODUCT_REQUIREMENTS_DOCUMENT.md`) between Full (max animation/interaction fidelity) and Performance (reduced pet FPS, reduced effects, lower-frequency UI animation) - implemented as a single setting read by both `zenpaws-pets` and the frontend's Motion usage, not two separately-maintained code paths.

## Measuring, not guessing

Phase 9 and Phase 10 use `cargo bench`/`criterion` for Rust-side hot paths (message insert/query, chunked transfer throughput) and manual profiling (browser devtools performance tab via the webview, plus OS-level CPU/RAM monitoring on the actual target-spec hardware where feasible) - see `20_TESTING_STRATEGY.md`.