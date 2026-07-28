use rusqlite::params;
use zenpaws_shared::PeerId;

use crate::{Database, DatabaseError, MessageRecord};

/// Lamport position used to request messages newer than a known event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LamportCursor {
    pub counter: i64,
    pub peer_id: PeerId,
}

impl Database {
    /// Inserts a replicated message idempotently by message ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the message cannot be persisted.
    pub fn insert_message_if_absent(&self, message: &MessageRecord) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT OR IGNORE INTO messages (
                id, room, author_uuid, body, reply_to, lamport_peer,
                lamport_counter, created_at, state_lamport_peer, state_lamport_counter
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?6, ?7)",
            params![
                message.id.to_string(),
                message.room,
                message.author.as_uuid().to_string(),
                message.body,
                message.reply_to.map(|id| id.to_string()),
                message.lamport_peer.as_uuid().to_string(),
                message.lamport_counter,
                message.created_at,
            ],
        )?;
        Ok(())
    }

    /// Returns replicated room messages newer than a Lamport cursor.
    ///
    /// # Errors
    ///
    /// Returns an error when the sync query or row decoding fails.
    pub fn messages_after_lamport(
        &self,
        room: &str,
        cursor: LamportCursor,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, DatabaseError> {
        let mut statement = self.connection.prepare(
            "SELECT id, room, author_uuid, body, reply_to, lamport_peer,
                    lamport_counter, created_at
             FROM messages
             WHERE room = ?1
               AND (lamport_counter > ?2 OR
                    (lamport_counter = ?2 AND lamport_peer > ?3))
             ORDER BY lamport_counter ASC, lamport_peer ASC
             LIMIT ?4",
        )?;
        let rows = statement.query_map(
            params![
                room,
                cursor.counter,
                cursor.peer_id.as_uuid().to_string(),
                i64::from(limit.clamp(1, 200)),
            ],
            crate::messages::row_to_message,
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

impl Database {
    /// Applies a remote edit only when its Lamport clock wins the visible state.
    ///
    /// # Errors
    ///
    /// Returns an error when the mutation cannot be stored.
    pub fn apply_remote_edit(
        &self,
        message_id: uuid::Uuid,
        body: &str,
        edited_at: i64,
        clock: LamportCursor,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "UPDATE messages
             SET body = ?1, edited_at = ?2,
                 state_lamport_peer = ?3, state_lamport_counter = ?4
             WHERE id = ?5
               AND (state_lamport_counter < ?4 OR
                    (state_lamport_counter = ?4 AND state_lamport_peer < ?3))",
            params![
                body,
                edited_at,
                clock.peer_id.as_uuid().to_string(),
                clock.counter,
                message_id.to_string(),
            ],
        )?;
        Ok(())
    }

    /// Applies a remote delete only when its Lamport clock wins the visible state.
    ///
    /// # Errors
    ///
    /// Returns an error when the mutation cannot be stored.
    pub fn apply_remote_delete(
        &self,
        message_id: uuid::Uuid,
        deleted_at: i64,
        clock: LamportCursor,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "UPDATE messages
             SET deleted_at = ?1,
                 state_lamport_peer = ?2, state_lamport_counter = ?3
             WHERE id = ?4
               AND (state_lamport_counter < ?3 OR
                    (state_lamport_counter = ?3 AND state_lamport_peer < ?2))",
            params![
                deleted_at,
                clock.peer_id.as_uuid().to_string(),
                clock.counter,
                message_id.to_string(),
            ],
        )?;
        Ok(())
    }
}

impl Database {
    /// Returns a peer display name for a frontend message projection.
    ///
    /// # Errors
    ///
    /// Returns an error when the peer query fails.
    pub fn peer_username(&self, peer_id: PeerId) -> Result<Option<String>, DatabaseError> {
        let result = self.connection.query_row(
            "SELECT username FROM peers WHERE uuid = ?1",
            [peer_id.as_uuid().to_string()],
            |row| row.get(0),
        );
        match result {
            Ok(username) => Ok(Some(username)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(DatabaseError::Sqlite(error)),
        }
    }
}

impl Database {
    /// Updates a replicated peer's display name without changing its trust pin.
    ///
    /// # Errors
    ///
    /// Returns an error when peer metadata cannot be updated.
    pub fn update_peer_username(
        &self,
        peer_id: PeerId,
        username: &str,
        last_seen_at: i64,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "UPDATE peers SET username = ?1, last_seen_at = ?2 WHERE uuid = ?3",
            params![username, last_seen_at, peer_id.as_uuid().to_string()],
        )?;
        Ok(())
    }
}

impl Database {
    /// Returns the emoji reactions stored for one message.
    ///
    /// # Errors
    ///
    /// Returns an error when reactions cannot be queried.
    pub fn reactions_for_message(
        &self,
        message_id: uuid::Uuid,
    ) -> Result<Vec<String>, DatabaseError> {
        let mut statement = self
            .connection
            .prepare("SELECT emoji FROM reactions WHERE message_id = ?1 ORDER BY emoji")?;
        let rows = statement.query_map([message_id.to_string()], |row| row.get(0))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

impl Database {
    /// Returns whether a message has been soft-deleted.
    ///
    /// # Errors
    ///
    /// Returns an error when the message state cannot be queried.
    pub fn message_is_deleted(&self, message_id: uuid::Uuid) -> Result<bool, DatabaseError> {
        let result = self.connection.query_row(
            "SELECT deleted_at IS NOT NULL FROM messages WHERE id = ?1",
            [message_id.to_string()],
            |row| row.get(0),
        );
        match result {
            Ok(deleted) => Ok(deleted),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(error) => Err(DatabaseError::Sqlite(error)),
        }
    }
}

impl Database {
    /// Toggles one peer's reaction and returns whether it was added.
    ///
    /// # Errors
    ///
    /// Returns an error when the reaction cannot be changed.
    pub fn toggle_reaction(
        &self,
        message_id: uuid::Uuid,
        peer_id: PeerId,
        emoji: &str,
    ) -> Result<bool, DatabaseError> {
        let deleted = self.connection.execute(
            "DELETE FROM reactions WHERE message_id = ?1 AND peer_uuid = ?2 AND emoji = ?3",
            params![message_id.to_string(), peer_id.as_uuid().to_string(), emoji],
        )?;
        if deleted > 0 {
            return Ok(false);
        }
        self.connection.execute(
            "INSERT INTO reactions (message_id, peer_uuid, emoji) VALUES (?1, ?2, ?3)",
            params![message_id.to_string(), peer_id.as_uuid().to_string(), emoji],
        )?;
        Ok(true)
    }
}

impl Database {
    /// Removes one peer reaction idempotently.
    ///
    /// # Errors
    ///
    /// Returns an error when the reaction cannot be removed.
    pub fn remove_reaction(
        &self,
        message_id: uuid::Uuid,
        peer_id: PeerId,
        emoji: &str,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "DELETE FROM reactions WHERE message_id = ?1 AND peer_uuid = ?2 AND emoji = ?3",
            params![message_id.to_string(), peer_id.as_uuid().to_string(), emoji],
        )?;
        Ok(())
    }
}

impl Database {
    /// Returns the author name and body for a reply target.
    pub fn reply_preview(
        &self,
        message_id: Option<uuid::Uuid>,
    ) -> Result<Option<(String, String)>, DatabaseError> {
        let Some(message_id) = message_id else {
            return Ok(None);
        };
        let result = self.connection.query_row(
            "SELECT p.username, m.body
             FROM messages AS m
             JOIN peers AS p ON p.uuid = m.author_uuid
             WHERE m.id = ?1",
            [message_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        match result {
            Ok(preview) => Ok(Some(preview)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(DatabaseError::Sqlite(error)),
        }
    }
}
