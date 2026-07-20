use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use zenpaws_shared::PeerId;

use crate::{Envelope, FrameError, read_envelope, write_envelope};

/// Local identity fields sent after the TLS handshake completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandshakeIdentity {
    peer_id: PeerId,
    username: String,
}

impl HandshakeIdentity {
    /// Validates fields that can safely appear in a LAN handshake.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty, oversized, or control-character username.
    pub fn new(peer_id: PeerId, username: String) -> Result<Self, HandshakeError> {
        if username.is_empty() || username.len() > 32 || username.chars().any(char::is_control) {
            return Err(HandshakeError::InvalidUsername);
        }
        Ok(Self { peer_id, username })
    }

    /// Returns this peer's stable identity.
    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Returns the validated display name received during the handshake.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }
}

/// Runs the initiator side of the encrypted application handshake.
///
/// # Errors
///
/// Returns an error for invalid protocol frames or unexpected remote identity.
pub async fn client_handshake<S>(
    stream: &mut S,
    local: &HandshakeIdentity,
    expected_peer: PeerId,
) -> Result<HandshakeIdentity, HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    send_handshake(stream, local).await?;
    receive_handshake(stream, expected_peer).await
}

/// Runs the accepting side of the encrypted application handshake.
///
/// # Errors
///
/// Returns an error for invalid protocol frames or an invalid remote username.
pub async fn server_handshake<S>(
    stream: &mut S,
    local: &HandshakeIdentity,
) -> Result<HandshakeIdentity, HandshakeError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let remote = receive_any_handshake(stream).await?;
    send_handshake(stream, local).await?;
    Ok(remote)
}

async fn send_handshake<S>(
    stream: &mut S,
    identity: &HandshakeIdentity,
) -> Result<(), HandshakeError>
where
    S: AsyncWrite + Unpin,
{
    write_envelope(
        stream,
        &Envelope::Handshake {
            peer_id: identity.peer_id,
            username: identity.username.clone(),
        },
    )
    .await?;
    Ok(())
}

async fn receive_handshake<S>(
    stream: &mut S,
    expected_peer: PeerId,
) -> Result<HandshakeIdentity, HandshakeError>
where
    S: AsyncRead + Unpin,
{
    let identity = receive_any_handshake(stream).await?;
    if identity.peer_id != expected_peer {
        return Err(HandshakeError::UnexpectedPeer);
    }
    Ok(identity)
}

async fn receive_any_handshake<S>(stream: &mut S) -> Result<HandshakeIdentity, HandshakeError>
where
    S: AsyncRead + Unpin,
{
    match read_envelope(stream).await? {
        Envelope::Handshake { peer_id, username } => HandshakeIdentity::new(peer_id, username),
        _ => Err(HandshakeError::ExpectedHandshake),
    }
}

/// Application handshake failures after TLS succeeds.
#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("handshake frame failed")]
    Frame(#[from] FrameError),
    #[error("handshake username is invalid")]
    InvalidUsername,
    #[error("expected a handshake envelope")]
    ExpectedHandshake,
    #[error("remote handshake peer identity did not match discovery")]
    UnexpectedPeer,
}

#[cfg(test)]
mod tests {
    use tokio::io::duplex;
    use zenpaws_shared::PeerId;

    use super::{HandshakeIdentity, client_handshake, server_handshake};

    #[tokio::test]
    async fn exchanges_identities_after_transport_security() {
        let client_id = PeerId::new();
        let server_id = PeerId::new();
        let client_identity =
            HandshakeIdentity::new(client_id, "client".to_owned()).expect("valid");
        let server_identity =
            HandshakeIdentity::new(server_id, "server".to_owned()).expect("valid");
        let (mut client_stream, mut server_stream) = duplex(4096);
        let server =
            tokio::spawn(
                async move { server_handshake(&mut server_stream, &server_identity).await },
            );

        let remote = client_handshake(&mut client_stream, &client_identity, server_id)
            .await
            .expect("valid remote identity");
        let received = server
            .await
            .expect("server task completes")
            .expect("server handshake");

        assert_eq!(remote.peer_id(), server_id);
        assert_eq!(received.peer_id(), client_id);
    }
}
