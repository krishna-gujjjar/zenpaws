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

impl MessageAckKind {
    const fn as_sql(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Read => "read",
        }
    }
}

impl Database {
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
