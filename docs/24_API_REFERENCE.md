# 24 - API Reference (Tauri Commands & Events)

Filled in progressively as each phase implements real commands - empty at Phase 1 since no feature commands exist yet beyond app scaffolding. This is the canonical list; `12_SECURITY_MODEL.md`'s capability files must stay in sync with whatever's added here.

## Format for each entry (used from Phase 2 onward)

```
### command_name
- **Window scope**: main-window | pet-* | both
- **Payload**: <TypeScript type>
- **Returns**: <TypeScript type>
- **Errors**: <enumerated error cases>
- **Added in**: Phase N
```

## Commands (none yet)

_(Phase 2+ will populate this as `zenpaws-network`, `zenpaws-database`, etc. expose their first real commands.)_

## Events (none yet)

_(Tauri events emitted from backend to frontend - e.g. `peer:connected`, `message:received` - documented here once Phase 2/5 introduce them.)_

### mark_message_read

- **Window scope**: main-window
- **Payload**: `{ messageId: string; peerId: string }`
- **Returns**: `void`
- **Errors**: invalid UUID, database lock, shared-room read receipt
- **Added in**: Phase 5