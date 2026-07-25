use rusqlite::params;
use thiserror::Error;
use uuid::Uuid;
use zenpaws_shared::PeerId;

use crate::Database;

/// Receipt kinds allowed by the hostless chat protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageAckKind {
    Delivered,
    Read,
}

/// Aggregate acknowledgement state for one message and recipient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageStatus {
    pub delivered: bool,
    pub read: bool,
}

impl MessageAckKind {
    const fn as_sql(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Read => "read",
        }
    }
}

impl Database {
    /// Returns delivery and read state for one recipient.
    ///
    /// # Errors
    ///
    /// Returns an error when acknowledgement data cannot be queried.
    pub fn message_status(
        &self,
        message_id: Uuid,
        peer_id: PeerId,
    ) -> Result<MessageStatus, MessageAckError> {
        let mut statement = self
            .connection
            .prepare("SELECT kind FROM message_acks WHERE message_id = ?1 AND peer_uuid = ?2")?;
        let rows = statement.query_map(
            params![message_id.to_string(), peer_id.as_uuid().to_string()],
            |row| row.get::<_, String>(0),
        )?;
        let mut status = MessageStatus {
            delivered: false,
            read: false,
        };
        for kind in rows {
            match kind?.as_str() {
                "delivered" => status.delivered = true,
                "read" => status.read = true,
                _ => {}
            }
        }
        Ok(status)
    }

    /// Stores an idempotent delivery or read acknowledgement.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown message, failed persistence, or a read
    /// acknowledgement for the shared room.
    pub fn acknowledge_message(
        &self,
        message_id: Uuid,
        peer_id: PeerId,
        kind: MessageAckKind,
        acked_at: i64,
    ) -> Result<(), MessageAckError> {
        let room: String = self.connection.query_row(
            "SELECT room FROM messages WHERE id = ?1",
            [message_id.to_string()],
            |row| row.get(0),
        )?;
        if kind == MessageAckKind::Read && room == "shared" {
            return Err(MessageAckError::SharedRoomRead);
        }
        self.connection.execute(
            "INSERT OR IGNORE INTO message_acks (message_id, peer_uuid, kind, acked_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                message_id.to_string(),
                peer_id.as_uuid().to_string(),
                kind.as_sql(),
                acked_at
            ],
        )?;
        Ok(())
    }
}

/// Message acknowledgement failures.
#[derive(Debug, Error)]
pub enum MessageAckError {
    #[error("shared-room messages do not have read receipts")]
    SharedRoomRead,
    #[error("message acknowledgement database operation failed")]
    Sqlite(#[from] rusqlite::Error),
}
