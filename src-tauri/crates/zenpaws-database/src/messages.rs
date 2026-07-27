use rusqlite::{params, types::Type};
use uuid::Uuid;
use zenpaws_shared::PeerId;

use crate::{Database, DatabaseError};

/// A persisted chat message independent of frontend representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageRecord {
    pub author: PeerId,
    pub body: String,
    pub created_at: i64,
    pub id: Uuid,
    pub lamport_counter: i64,
    pub lamport_peer: PeerId,
    pub reply_to: Option<Uuid>,
    pub room: String,
}

/// Stable keyset cursor for descending message history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageCursor {
    pub created_at: i64,
    pub id: Uuid,
}

impl Database {
    /// Inserts one message through a prepared statement.
    ///
    /// # Errors
    ///
    /// Returns an error when foreign keys or message constraints fail.
    pub fn insert_message(&self, message: &MessageRecord) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT INTO messages (
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

    /// Returns one room page using a keyset cursor rather than `OFFSET`.
    ///
    /// # Errors
    ///
    /// Returns an error when the query or row decoding fails.
    pub fn messages_before(
        &self,
        room: &str,
        cursor: Option<&MessageCursor>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, DatabaseError> {
        let limit = i64::from(limit.clamp(1, 200));
        let mut messages = Vec::new();
        if let Some(cursor) = cursor {
            let mut statement = self.connection.prepare(
                "SELECT id, room, author_uuid, body, reply_to, lamport_peer,
                        lamport_counter, created_at
                 FROM messages
                 WHERE room = ?1
                   AND (created_at < ?2 OR (created_at = ?2 AND id < ?3))
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?4",
            )?;
            let rows = statement.query_map(
                params![room, cursor.created_at, cursor.id.to_string(), limit],
                row_to_message,
            )?;
            messages.extend(rows.collect::<Result<Vec<_>, _>>()?);
        } else {
            let mut statement = self.connection.prepare(
                "SELECT id, room, author_uuid, body, reply_to, lamport_peer,
                            lamport_counter, created_at
                     FROM messages
                     WHERE room = ?1
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?2",
            )?;
            let rows = statement.query_map(params![room, limit], row_to_message)?;
            messages.extend(rows.collect::<Result<Vec<_>, _>>()?);
        }
        Ok(messages)
    }

    /// Edits a message body and records the edit timestamp.
    ///
    /// # Errors
    ///
    /// Returns an error when the message update fails.
    pub fn edit_message(&self, id: Uuid, body: &str, edited_at: i64) -> Result<(), DatabaseError> {
        self.connection.execute(
            "UPDATE messages SET body = ?1, edited_at = ?2 WHERE id = ?3",
            params![body, edited_at, id.to_string()],
        )?;
        Ok(())
    }

    /// Marks a message deleted while retaining replication history.
    ///
    /// # Errors
    ///
    /// Returns an error when the message update fails.
    pub fn delete_message(&self, id: Uuid, deleted_at: i64) -> Result<(), DatabaseError> {
        self.connection.execute(
            "UPDATE messages SET deleted_at = ?1 WHERE id = ?2",
            params![deleted_at, id.to_string()],
        )?;
        Ok(())
    }

    /// Adds one unique emoji reaction through a prepared statement.
    ///
    /// # Errors
    ///
    /// Returns an error when reaction persistence fails.
    pub fn add_reaction(
        &self,
        message_id: Uuid,
        peer_id: PeerId,
        emoji: &str,
    ) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT OR IGNORE INTO reactions (message_id, peer_uuid, emoji) VALUES (?1, ?2, ?3)",
            params![message_id.to_string(), peer_id.as_uuid().to_string(), emoji],
        )?;
        Ok(())
    }

    /// Searches locally indexed message text.
    ///
    /// # Errors
    ///
    /// Returns an error when the FTS query or row decoding fails.
    pub fn search_messages(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, DatabaseError> {
        let mut statement = self.connection.prepare(
            "SELECT m.id, m.room, m.author_uuid, m.body, m.reply_to, m.lamport_peer,
                    m.lamport_counter, m.created_at
             FROM messages_fts
             JOIN messages AS m ON m.rowid = messages_fts.rowid
             WHERE messages_fts MATCH ?1
             ORDER BY m.created_at DESC, m.id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(
            params![query, i64::from(limit.clamp(1, 100))],
            row_to_message,
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

pub fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRecord> {
    let parse_uuid = |index| {
        row.get::<_, String>(index).and_then(|value| {
            Uuid::parse_str(&value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
            })
        })
    };
    Ok(MessageRecord {
        id: parse_uuid(0)?,
        room: row.get(1)?,
        author: PeerId::from_uuid(parse_uuid(2)?),
        body: row.get(3)?,
        reply_to: row
            .get::<_, Option<String>>(4)?
            .map(|value| Uuid::parse_str(&value))
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(error))
            })?,
        lamport_peer: PeerId::from_uuid(parse_uuid(5)?),
        lamport_counter: row.get(6)?,
        created_at: row.get(7)?,
    })
}
