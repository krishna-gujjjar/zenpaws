# 07 - Message Protocol

## Wire format

```
┌──────────────┬───────────────────┬──────────────────────────┐
│ length (u32) │ protocol_ver (u8) │ bincode(serde enum body)  │
│ 4 bytes, LE  │ 1 byte            │ `length` bytes            │
└──────────────┴───────────────────┴──────────────────────────┘
```

Length-prefixed framing over the TCP stream from `06_NETWORK_ARCHITECTURE.md`.
`bincode` chosen over JSON for this payload - see `05_TECHNICAL_DECISIONS.md`
for why.

## Envelope body (conceptual shape - refined in Phase 5 implementation)

```rust
enum Envelope {
    Handshake { peer_uuid: Uuid, username: String, pubkey_fingerprint: [u8; 32] },
    Heartbeat,
    Message(MessagePayload),
    Ack { message_id: Uuid, kind: AckKind }, // Delivered | Read
    PetState(PetStateEvent),                  // see 09_PET_ENGINE_SPECIFICATION.md
    TransferControl(TransferMessage),          // see 08_FILE_TRANSFER_PROTOCOL.md
    SyncRequest { since_clock: LamportClock }, // see 27_SYNC_REPLICATION_MODEL.md
}

struct MessagePayload {
    id: Uuid,
    room: RoomTarget,       // SharedRoom | Direct(Uuid)
    author: Uuid,
    body: MessageBody,      // Text | Edit | Delete | Reaction
    clock: LamportClock,    // (peer_uuid, counter) - see 27
    created_at: i64,        // unix millis, local clock - not trusted for ordering
    reply_to: Option<Uuid>,
    mentions: Vec<Uuid>,
}
```

## Delivery/read status

Implements the semantics defined in `06_NETWORK_ARCHITECTURE.md` via the
`Ack` variant above. `Read` acknowledgements are valid for direct messages
only; shared-room messages do not have per-peer read state.

## Edit / delete / conflict handling

An edit or delete is itself a `MessagePayload` referencing the original
`id`. When two peers edit the same message concurrently (rare on a small
LAN room, but possible), the Lamport-clock last-write-wins rule from
`27_SYNC_REPLICATION_MODEL.md` decides which edit wins; the losing edit is
kept in local history (not silently dropped) so it can be surfaced if ever
needed for debugging, but not shown in the primary thread.

## Search

Full-text search is local-only (SQLite FTS5 against the local replica of
message history) - it is never a network operation. See
`09_DATABASE_SCHEMA.md`.
## Phase 5 implementation status

`MessagePayload` and `MessageBody` are now implemented in
`zenpaws-network/src/protocol.rs`. A locally persisted text message is wrapped
in `Envelope::Message`, queued to all connected peer write channels, and a
receiving peer sends an `Ack::Delivered` after accepting the frame. The Tauri
application persists received text messages through the shared event bus.

Edit, delete, reaction, and read-ack envelopes remain the next network
mutation step; their local database commands already exist.
