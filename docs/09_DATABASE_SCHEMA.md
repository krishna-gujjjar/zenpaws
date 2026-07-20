# 09 - Database Schema

SQLite, WAL mode, FTS5, prepared statements only (no string-concatenated
SQL, ever - this is also a security requirement, see `12_SECURITY_MODEL.md`).
Accessed exclusively through `zenpaws-database`'s typed query functions;
never raw SQL from the frontend (see the `rusqlite` decision in
`05_TECHNICAL_DECISIONS.md`).

## Migrations

Managed as sequential, never-edited files (`migrations/0001_init.sql`,
`0002_...`). **Never edit an existing migration** - always add a new one,
even to fix a mistake in an earlier one. This is what keeps every peer's
schema reconstructable from history.

## Core tables (initial shape - refined in Phase 3 implementation)

```sql
CREATE TABLE peers (
    uuid TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    avatar_path TEXT,
    pubkey_fingerprint BLOB NOT NULL,   -- TOFU pin, see 12_SECURITY_MODEL.md
    last_seen_at INTEGER NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,                -- UUID
    room TEXT NOT NULL,                 -- 'shared' or a DM peer UUID
    author_uuid TEXT NOT NULL REFERENCES peers(uuid),
    body TEXT NOT NULL,
    reply_to TEXT REFERENCES messages(id),
    lamport_peer TEXT NOT NULL,         -- Lamport clock, see 27_SYNC_REPLICATION_MODEL.md
    lamport_counter INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    edited_at INTEGER,
    deleted_at INTEGER
);

CREATE VIRTUAL TABLE messages_fts USING fts5(
    body, content='messages', content_rowid='rowid'
);

CREATE TABLE reactions (
    message_id TEXT NOT NULL REFERENCES messages(id),
    peer_uuid TEXT NOT NULL REFERENCES peers(uuid),
    emoji TEXT NOT NULL,
    PRIMARY KEY (message_id, peer_uuid, emoji)
);

CREATE TABLE message_acks (
    message_id TEXT NOT NULL REFERENCES messages(id),
    peer_uuid TEXT NOT NULL REFERENCES peers(uuid),
    kind TEXT NOT NULL CHECK (kind IN ('delivered', 'read')),
    acked_at INTEGER NOT NULL,
    PRIMARY KEY (message_id, peer_uuid, kind)
);

CREATE TABLE files (
    id TEXT PRIMARY KEY,                -- fileId (UUID)
    message_id TEXT NOT NULL REFERENCES messages(id),
    sender_uuid TEXT NOT NULL REFERENCES peers(uuid),
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    checksum_sha256 BLOB NOT NULL,
    original_filename TEXT NOT NULL,
    stored_filename TEXT NOT NULL,      -- UUID-based, sanitized
    created_at INTEGER NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL                 -- JSON-encoded
);
```

## Indexing

- `messages(room, created_at)` - the hot path for scrollback pagination.
- `messages(lamport_peer, lamport_counter)` - sync/conflict resolution
  lookups (`27_SYNC_REPLICATION_MODEL.md`).
- FTS5's own index covers search; no separate search index needed.

## Why these choices satisfy the 100k-message requirement

Pagination queries against `messages(room, created_at)` with `LIMIT`/keyset
pagination (not `OFFSET`, which degrades at scale) plus the virtualized list
on the frontend (`@tanstack/react-virtual`) are what make 100,000+ messages
scroll smoothly - this is a combined DB + UI concern, cross-reference
`13_PERFORMANCE_GUIDELINES.md`.
