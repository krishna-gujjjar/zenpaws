use std::env;

use super::{Database, MessageAckKind, MessageRecord};
use uuid::Uuid;
use zenpaws_shared::PeerId;

#[test]
fn applies_initial_migration_and_writes_a_peer() {
    let database = Database::open_in_memory().expect("database opens");
    database
        .upsert_peer(PeerId::new(), "peer", &[7; 32], 1)
        .expect("peer writes");

    let peer_id = PeerId::new();
    database
        .upsert_peer(peer_id, "author", &[8; 32], 2)
        .expect("author writes");
    database
        .insert_message(&MessageRecord {
            author: peer_id,
            body: "searchable local message".to_owned(),
            created_at: 2,
            id: Uuid::new_v4(),
            lamport_counter: 1,
            lamport_peer: peer_id,
            reply_to: None,
            room: "shared".to_owned(),
        })
        .expect("message writes");

    assert_eq!(database.migration_count().expect("migrations query"), 4);
    assert_eq!(
        database
            .search_messages("searchable", 10)
            .expect("fts query")
            .len(),
        1
    );
    database
        .set_setting("theme", "\"dark\"")
        .expect("setting writes");
    assert_eq!(
        database.setting("theme").expect("setting reads"),
        Some("\"dark\"".to_owned())
    );
}

#[test]
fn database_can_be_owned_by_tauri_mutex_state() {
    fn assert_send<T: Send>() {}
    assert_send::<Database>();
}

#[test]
fn configures_wal_and_reuses_migrations_on_disk() {
    let path = env::temp_dir().join(format!("zenpaws-db-{}.sqlite", Uuid::new_v4()));
    let database = Database::open(&path).expect("database opens");
    let journal_mode: String = database
        .connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("journal mode reads");
    assert_eq!(journal_mode, "wal");
    drop(database);

    let reopened = Database::open(&path).expect("database reopens");
    assert_eq!(reopened.migration_count().expect("migrations query"), 4);
    drop(reopened);
    std::fs::remove_file(&path).expect("database file is removable");
    let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
    let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
}

#[test]
fn applies_chat_mutations_and_receipts_to_one_replica() {
    let database = Database::open_in_memory().expect("database opens");
    let author = PeerId::new();
    let recipient = PeerId::new();
    let message_id = Uuid::new_v4();
    database
        .upsert_peer(author, "author", &[1; 32], 1)
        .expect("author writes");
    database
        .upsert_peer(recipient, "recipient", &[2; 32], 1)
        .expect("recipient writes");
    database
        .insert_message(&MessageRecord {
            author,
            body: "before mutation".to_owned(),
            created_at: 1,
            id: message_id,
            lamport_counter: 1,
            lamport_peer: author,
            reply_to: None,
            room: format!("dm:{}", recipient.as_uuid()),
        })
        .expect("message writes");

    database
        .edit_message(message_id, "after mutation", 2)
        .expect("edit applies");
    database
        .add_reaction(message_id, recipient, "👍")
        .expect("reaction applies");
    database
        .acknowledge_message(message_id, recipient, MessageAckKind::Delivered, 3)
        .expect("delivery receipt applies");
    database
        .acknowledge_message(message_id, recipient, MessageAckKind::Read, 4)
        .expect("read receipt applies");
    database
        .delete_message(message_id, 5)
        .expect("delete applies");

    let status = database
        .message_status(message_id, recipient)
        .expect("receipt status reads");
    assert!(status.delivered);
    assert!(status.read);
}

#[test]
fn replicated_message_insert_is_idempotent() {
    let database = Database::open_in_memory().expect("database opens");
    let author = PeerId::new();
    let message = MessageRecord {
        author,
        body: "replicated once".to_owned(),
        created_at: 1,
        id: Uuid::new_v4(),
        lamport_counter: 1,
        lamport_peer: author,
        reply_to: None,
        room: "shared".to_owned(),
    };
    database
        .upsert_peer(author, "remote", &[3; 32], 1)
        .expect("peer writes");
    database
        .insert_message_if_absent(&message)
        .expect("first replica insert");
    database
        .insert_message_if_absent(&message)
        .expect("duplicate replica insert is ignored");

    assert_eq!(
        database
            .messages_before("shared", None, 10)
            .expect("messages read")
            .len(),
        1
    );
}

#[test]
fn reads_messages_after_a_lamport_cursor_for_reconnect_sync() {
    let database = Database::open_in_memory().expect("database opens");
    let author = PeerId::new();
    database
        .upsert_peer(author, "remote", &[4; 32], 1)
        .expect("peer writes");
    for counter in 1..=3 {
        database
            .insert_message(&MessageRecord {
                author,
                body: format!("message {counter}"),
                created_at: counter,
                id: Uuid::new_v4(),
                lamport_counter: counter,
                lamport_peer: author,
                reply_to: None,
                room: "shared".to_owned(),
            })
            .expect("message writes");
    }

    let messages = database
        .messages_after_lamport(
            "shared",
            super::LamportCursor {
                counter: 1,
                peer_id: author,
            },
            10,
        )
        .expect("sync query succeeds");
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].lamport_counter, 2);
    assert_eq!(messages[1].lamport_counter, 3);
}

#[test]
fn rejects_an_older_remote_edit_after_a_newer_one() {
    let database = Database::open_in_memory().expect("database opens");
    let author = PeerId::new();
    let message_id = Uuid::new_v4();
    database
        .upsert_peer(author, "author", &[5; 32], 1)
        .expect("peer writes");
    database
        .insert_message(&MessageRecord {
            author,
            body: "initial".to_owned(),
            created_at: 1,
            id: message_id,
            lamport_counter: 1,
            lamport_peer: author,
            reply_to: None,
            room: "shared".to_owned(),
        })
        .expect("message writes");
    database
        .apply_remote_edit(
            message_id,
            "newer",
            3,
            super::LamportCursor {
                counter: 3,
                peer_id: author,
            },
        )
        .expect("newer edit applies");
    database
        .apply_remote_edit(
            message_id,
            "older",
            2,
            super::LamportCursor {
                counter: 2,
                peer_id: author,
            },
        )
        .expect("older edit is safely ignored");

    let message = database
        .messages_before("shared", None, 10)
        .expect("message reads")
        .pop()
        .expect("message exists");
    assert_eq!(message.body, "newer");
}
