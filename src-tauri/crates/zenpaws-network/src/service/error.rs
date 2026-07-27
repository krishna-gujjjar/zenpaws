use thiserror::Error;

use crate::{FrameError, HandshakeError, TlsError, TransportError};

/// Network-service lifecycle failures.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("TCP listener failed")]
    Io(#[from] std::io::Error),
    #[error("mDNS lifecycle failed")]
    Discovery(#[from] crate::DiscoveryError),
    #[error("UDP fallback discovery failed")]
    Udp(#[from] crate::UdpError),
    #[error("TLS configuration or handshake failed")]
    Tls(#[from] TlsError),
    #[error("TLS transport failed")]
    Transport(#[from] TransportError),
    #[error("application handshake failed")]
    Handshake(#[from] HandshakeError),
    #[error("peer protocol framing failed")]
    Frame(#[from] FrameError),
    #[error("peer state update failed")]
    State(#[from] crate::StateError),
    #[error("discovered peer attempted to connect to itself")]
    SelfConnection,
    #[error("discovered peer TLS server name was invalid")]
    InvalidServerName,
    #[error("peer registry lock was poisoned")]
    PeerRegistryPoisoned,
    #[error("peer is not connected")]
    PeerNotConnected,
    #[error("peer outbound channel is closed")]
    PeerChannelClosed,
}
