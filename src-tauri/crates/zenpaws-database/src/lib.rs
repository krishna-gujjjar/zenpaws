//! `SQLite` persistence, migrations, WAL mode, and typed database operations.
//!
//! See `docs/09_DATABASE_SCHEMA.md`.

use std::{fs, path::Path};

use rusqlite::{Connection, OpenFlags, params};
use thiserror::Error;
use zenpaws_shared::PeerId;

mod acks;
mod messages;
mod replication;
mod trust;

pub use acks::{MessageAckError, MessageAckKind, MessageStatus};
pub use messages::{MessageCursor, MessageRecord};
pub use replication::LamportCursor;
pub use trust::DatabaseTrustStore;

const MIGRATIONS: [(&str, &str); 4] = [
    ("0001_init", include_str!("../migrations/0001_init.sql")),
    (
        "0002_messages_fts",
        include_str!("../migrations/0002_messages_fts.sql"),
    ),
    (
        "0003_peer_certificates",
        include_str!("../migrations/0003_peer_certificates.sql"),
    ),
    (
        "0004_message_lww",
        include_str!("../migrations/0004_message_lww.sql"),
    ),
];

/// A typed `SQLite` database connection owned by the database crate.
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens a database, enables required `SQLite` pragmas, and applies migrations.
    ///
    /// # Errors
    ///
    /// Returns an error when `SQLite` cannot open, configure, migrate, or validate
    /// the local database.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        Self::configure(&connection)?;
        Self::migrate(&connection)?;
        Ok(Self { connection })
    }

    /// Creates an isolated in-memory database for tests.
    ///
    /// # Errors
    ///
    /// Returns an error when `SQLite` configuration or migration fails.
    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        let connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Self::migrate(&connection)?;
        Ok(Self { connection })
    }

    /// Upserts peer metadata while preserving prepared-statement boundaries.
    ///
    /// # Errors
    ///
    /// Returns an error if peer metadata cannot be written.
    pub fn upsert_peer(
        &self,
        peer_id: PeerId,
        username: &str,
        fingerprint: &[u8],
        last_seen_at: i64,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT INTO peers (uuid, username, pubkey_fingerprint, last_seen_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(uuid) DO UPDATE SET
               username = excluded.username,
               pubkey_fingerprint = excluded.pubkey_fingerprint,
               last_seen_at = excluded.last_seen_at",
            params![
                peer_id.as_uuid().to_string(),
                username,
                fingerprint,
                last_seen_at
            ],
        )?;
        Ok(())
    }

    /// Upserts one JSON-encoded application setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the setting cannot be written.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Returns a JSON-encoded application setting when it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the setting query fails.
    pub fn setting(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let value =
            self.connection
                .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                    row.get(0)
                });
        match value {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(DatabaseError::Sqlite(error)),
        }
    }

    /// Returns the applied migration count.
    ///
    /// # Errors
    ///
    /// Returns an error if migration metadata cannot be queried.
    pub fn migration_count(&self) -> Result<u32, DatabaseError> {
        let count =
            self.connection
                .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                    row.get::<_, u32>(0)
                })?;
        Ok(count)
    }

    fn configure(connection: &Connection) -> Result<(), DatabaseError> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(())
    }

    fn migrate(connection: &Connection) -> Result<(), DatabaseError> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 name TEXT PRIMARY KEY,
                 applied_at INTEGER NOT NULL
             );",
        )?;
        for (name, sql) in MIGRATIONS {
            let applied = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = ?1)",
                [name],
                |row| row.get::<_, bool>(0),
            )?;
            if !applied {
                let transaction = connection.unchecked_transaction()?;
                transaction.execute_batch(sql)?;
                transaction.execute(
                    "INSERT INTO schema_migrations (name, applied_at) VALUES (?1, unixepoch())",
                    [name],
                )?;
                transaction.commit()?;
            }
        }
        Ok(())
    }
}

/// Database failures exposed by typed database functions.
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database path I/O failed")]
    Io(#[from] std::io::Error),
    #[error("`SQLite` operation failed")]
    Sqlite(#[from] rusqlite::Error),
}

#[cfg(test)]
mod tests;
