use std::io;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;
use zenpaws_shared::PeerId;

/// Independent protocol version for wire compatibility checks.
pub const PROTOCOL_VERSION: u8 = 1;
/// Upper bound for one framed control message.
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Acknowledgement kinds permitted by the receipt policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AckKind {
    Delivered,
    Read,
}

/// Lamport ordering metadata attached to every replicated message mutation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LamportClock {
    pub counter: i64,
    pub peer_id: PeerId,
}

impl LamportClock {
    /// Returns whether this clock wins the deterministic LWW comparison.
    #[must_use]
    pub fn is_newer_than(self, other: Self) -> bool {
        (self.counter, self.peer_id.as_uuid()) > (other.counter, other.peer_id.as_uuid())
    }
}

/// Message content carried by the Phase 5 wire envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageBody {
    Text(String),
    Edit { message_id: Uuid, body: String },
    Delete { message_id: Uuid },
    Reaction {
        message_id: Uuid,
        emoji: String,
        removed: bool,
    },
}

/// A hostless replicated chat message or message mutation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessagePayload {
    pub author: PeerId,
    pub body: MessageBody,
    pub clock: LamportClock,
    pub created_at: i64,
    pub id: Uuid,
    pub mentions: Vec<PeerId>,
    pub reply_to: Option<Uuid>,
    pub room: String,
}

/// The versioned control and chat envelope exchanged by LAN peers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Envelope {
    Handshake { peer_id: PeerId, username: String },
    Heartbeat,
    Message(MessagePayload),
    SyncRequest { room: String, since: LamportClock },
    Ack { message_id: Uuid, kind: AckKind },
}

/// Protocol framing and serialization failures.
#[derive(Debug, Error)]
pub enum FrameError {
    #[error("protocol version {received} is unsupported")]
    UnsupportedVersion { received: u8 },
    #[error("frame is {received} bytes, over the {maximum}-byte limit")]
    FrameTooLarge { received: usize, maximum: usize },
    #[error("failed to serialize or deserialize the frame")]
    Serialization(#[from] Box<bincode::ErrorKind>),
    #[error("I/O failed while framing a peer message")]
    Io(#[from] io::Error),
}

/// Writes one versioned, length-prefixed envelope.
///
/// # Errors
///
/// Returns an error if serialization fails, the encoded frame exceeds the
/// documented limit, or the stream cannot be written.
pub async fn write_envelope<W>(writer: &mut W, envelope: &Envelope) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
{
    let body = bincode::serialize(envelope)?;
    if body.len() > MAX_FRAME_BYTES {
        return Err(FrameError::FrameTooLarge {
            received: body.len(),
            maximum: MAX_FRAME_BYTES,
        });
    }

    writer
        .write_u32_le(
            u32::try_from(body.len()).map_err(|_| FrameError::FrameTooLarge {
                received: body.len(),
                maximum: MAX_FRAME_BYTES,
            })?,
        )
        .await?;
    writer.write_u8(PROTOCOL_VERSION).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

/// Reads one versioned, length-prefixed envelope.
///
/// # Errors
///
/// Returns an error for unsupported peers, oversized input, malformed frames,
/// or a failed stream read.
pub async fn read_envelope<R>(reader: &mut R) -> Result<Envelope, FrameError>
where
    R: AsyncRead + Unpin,
{
    let length = reader.read_u32_le().await? as usize;
    if length > MAX_FRAME_BYTES {
        return Err(FrameError::FrameTooLarge {
            received: length,
            maximum: MAX_FRAME_BYTES,
        });
    }

    let version = reader.read_u8().await?;
    if version != PROTOCOL_VERSION {
        return Err(FrameError::UnsupportedVersion { received: version });
    }

    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    Ok(bincode::deserialize(&body)?)
}

#[cfg(test)]
mod tests {
    use tokio::io::duplex;
    use uuid::Uuid;
    use zenpaws_shared::PeerId;

    use super::{
        AckKind, Envelope, LamportClock, MessageBody, MessagePayload, read_envelope, write_envelope,
    };

    #[tokio::test]
    async fn round_trips_a_message_envelope() {
        let (mut writer, mut reader) = duplex(1024);
        let peer_id = PeerId::new();
        let envelope = Envelope::Message(MessagePayload {
            author: peer_id,
            body: MessageBody::Text("hello".to_owned()),
            clock: LamportClock {
                counter: 1,
                peer_id,
            },
            created_at: 1,
            id: Uuid::new_v4(),
            mentions: Vec::new(),
            reply_to: None,
            room: "shared".to_owned(),
        });

        let writer_task = tokio::spawn(async move { write_envelope(&mut writer, &envelope).await });
        let received = read_envelope(&mut reader).await.expect("frame is valid");
        writer_task
            .await
            .expect("writer task completes")
            .expect("frame writes");

        assert!(matches!(received, Envelope::Message(_)));
    }

    #[tokio::test]
    async fn round_trips_a_sync_request() {
        let (mut writer, mut reader) = duplex(1024);
        let peer_id = PeerId::new();
        let envelope = Envelope::SyncRequest {
            room: "shared".to_owned(),
            since: LamportClock {
                counter: 4,
                peer_id,
            },
        };

        let writer_task = tokio::spawn(async move { write_envelope(&mut writer, &envelope).await });
        let received = read_envelope(&mut reader).await.expect("sync frame reads");
        writer_task
            .await
            .expect("writer task completes")
            .expect("sync frame writes");

        assert!(matches!(received, Envelope::SyncRequest { .. }));
    }

    #[tokio::test]
    async fn round_trips_an_ack_envelope() {
        let (mut writer, mut reader) = duplex(1024);
        let envelope = Envelope::Ack {
            message_id: Uuid::new_v4(),
            kind: AckKind::Delivered,
        };

        let writer_task = tokio::spawn(async move { write_envelope(&mut writer, &envelope).await });
        let received = read_envelope(&mut reader).await.expect("frame is valid");
        writer_task
            .await
            .expect("writer task completes")
            .expect("frame writes");

        assert!(matches!(
            received,
            Envelope::Ack {
                kind: AckKind::Delivered,
                ..
            }
        ));
    }

    #[test]
    fn orders_equal_counter_clocks_by_peer_id() {
        let lower = LamportClock {
            counter: 7,
            peer_id: PeerId::from_uuid(Uuid::from_u128(1)),
        };
        let higher = LamportClock {
            counter: 7,
            peer_id: PeerId::from_uuid(Uuid::from_u128(2)),
        };

        assert!(higher.is_newer_than(lower));
        assert!(!lower.is_newer_than(higher));
    }
}
