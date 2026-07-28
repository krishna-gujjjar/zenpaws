# 10 - Pet Engine Specification

## Window architecture

Each active pet is its own transparent, always-on-top, click-through-capable Tauri window (`pet-<slug>-<instance>`), matched against the `pet-*` capability scope in `12_SECURITY_MODEL.md` - minimal IPC surface, no filesystem/network access from that window's webview context.

The pet engine (`zenpaws-pets`) must keep running even if the chat system is disabled - it subscribes to chat-originated events via the Event Bus but has no compile-time or runtime dependency on `zenpaws-network`'s internals.

## Data-driven pet definition

```
assets/pets/<pet>/
  manifest.json
  sprites/
  sounds/
  LICENSE.txt          -- required; see 23_ASSET_LICENSES.md
```

`manifest.json` shape (refined during Phase 4 implementation):

```json
{
  "id": "cat",
  "displayName": "Cat",
  "states": {
    "idle": { "sprite": "sprites/idle.png", "frameCount": 4, "fps": 6 },
    "walk": { "sprite": "sprites/walk.png", "frameCount": 6, "fps": 10 }
  },
  "sounds": { "notification": "sounds/notify.wav" },
  "defaultScale": 1.0
}
```

No hardcoded pets in application code - adding a pet is adding a directory that satisfies this schema, validated at load time.

## States

`idle`, `walk`, `run`, `jump`, `sleep`, `typing`, `notification`, `celebrate`, `fight`, `play`, `away`, `offline`.

## Behaviors

Chase cursor, react to rapid cursor movement, interact with other local pets, celebrate on new messages, display chat notifications, idle wandering. All behaviors are local-only in this build - see Synchronization below for what is and isn't networked.

## Synchronization (future: one pet per LAN user)

**Never synchronized**: coordinates, frame positions, per-frame animation state - this would be bandwidth-expensive and pointless at 50 peers.

**Synchronized** (discrete events only): `typing`, `playing`, `sleeping`, `celebrating`, `notification`, `away`, `offline` - carried as `PetStateEvent` in the message envelope (`07_MESSAGE_PROTOCOL.md`).

**Rate limit: max 2 broadcasts/sec/pet**, with debounce (don't re-send the same state repeatedly) and deduplication (a receiving peer ignores a duplicate event) implemented in `zenpaws-pets`, not left to network-layer throttling alone.

## Performance budget

< 1% CPU per active pet on the target hardware (`13_PERFORMANCE_GUIDELINES.md`). Performance Mode (`26_STORAGE_STRATEGY.md`'s sibling concept in `13_PERFORMANCE_GUIDELINES.md`) reduces pet FPS and effect complexity when enabled.