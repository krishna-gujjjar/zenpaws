use std::env;

use super::{Database, MessageRecord};
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

    assert_eq!(database.migration_count().expect("migrations query"), 3);
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
    assert_eq!(reopened.migration_count().expect("migrations query"), 3);
    drop(reopened);
    std::fs::remove_file(&path).expect("database file is removable");
    let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
    let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));
}
