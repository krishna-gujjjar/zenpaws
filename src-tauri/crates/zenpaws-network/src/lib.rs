//! LAN discovery, framed TCP protocol, and peer connection state.
//!
//! See `docs/06_NETWORK_ARCHITECTURE.md` and `docs/07_MESSAGE_PROTOCOL.md`.

mod backoff;
mod discovery;
mod protocol;
mod service;
mod session;
mod state;
mod tls;
mod transport;
mod udp;

pub use backoff::RetryBackoff;
pub use discovery::{DiscoveredPeer, DiscoveryError, MdnsDiscovery, SERVICE_TYPE};
pub use protocol::{AckKind, Envelope, FrameError, MAX_FRAME_BYTES, read_envelope, write_envelope};
pub use service::{NetworkService, PeerConnection, ServiceError};
pub use session::{HandshakeError, HandshakeIdentity, client_handshake, server_handshake};
pub use state::{ConnectionStateMachine, StateError};
pub use tls::{TlsError, TlsIdentity};
pub use transport::{TransportError, accept_tls, connect_tls};
pub use udp::{
    MAX_DISCOVERY_DATAGRAM_BYTES, UDP_DISCOVERY_PORT, UdpAdvertisement, UdpDiscovery, UdpError,
};
