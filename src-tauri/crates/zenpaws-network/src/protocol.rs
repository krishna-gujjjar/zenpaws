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

/// The small control envelope used before chat payloads are added in Phase 5.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Envelope {
    Handshake { peer_id: PeerId, username: String },
    Heartbeat,
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

    use super::{AckKind, Envelope, read_envelope, write_envelope};

    #[tokio::test]
    async fn round_trips_an_envelope() {
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
}
