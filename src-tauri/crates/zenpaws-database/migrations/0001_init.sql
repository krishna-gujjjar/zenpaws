CREATE TABLE peers (
    uuid TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    avatar_path TEXT,
    pubkey_fingerprint BLOB NOT NULL,
    last_seen_at INTEGER NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    room TEXT NOT NULL,
    author_uuid TEXT NOT NULL REFERENCES peers(uuid),
    body TEXT NOT NULL,
    reply_to TEXT REFERENCES messages(id),
    lamport_peer TEXT NOT NULL,
    lamport_counter INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    edited_at INTEGER,
    deleted_at INTEGER
);

CREATE VIRTUAL TABLE messages_fts USING fts5(
    body,
    content='messages',
    content_rowid='rowid'
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
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id),
    sender_uuid TEXT NOT NULL REFERENCES peers(uuid),
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    checksum_sha256 BLOB NOT NULL,
    original_filename TEXT NOT NULL,
    stored_filename TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX messages_room_created_at ON messages(room, created_at DESC, id DESC);
CREATE INDEX messages_lamport ON messages(lamport_peer, lamport_counter);
