use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::net::UdpSocket;
use zenpaws_shared::PeerId;

use crate::protocol::PROTOCOL_VERSION;

/// Upper bound for one UDP discovery datagram.
pub const MAX_DISCOVERY_DATAGRAM_BYTES: usize = 1024;
/// Default LAN broadcast port for the mDNS fallback.
pub const UDP_DISCOVERY_PORT: u16 = 46_235;

/// Metadata announced when multicast DNS is unavailable.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UdpAdvertisement {
    /// Public self-signed certificate DER for TOFU pinning.
    pub certificate_der: Vec<u8>,
    /// Advertised TCP listener port.
    pub tcp_port: u16,
    /// Stable peer identity.
    pub peer_id: PeerId,
    /// Wire compatibility version.
    pub protocol_version: u8,
    /// Display name for discovery UI.
    pub username: String,
}

impl UdpAdvertisement {
    /// Creates a versioned advertisement after validating its display metadata.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty, oversized, or control-character username.
    pub fn new(
        peer_id: PeerId,
        username: String,
        tcp_port: u16,
        certificate_der: Vec<u8>,
    ) -> Result<Self, UdpError> {
        if username.is_empty() || username.len() > 32 || username.chars().any(char::is_control) {
            return Err(UdpError::InvalidUsername);
        }
        Ok(Self {
            certificate_der,
            tcp_port,
            peer_id,
            protocol_version: PROTOCOL_VERSION,
            username,
        })
    }
}

/// Bounded UDP broadcast fallback for LAN peer discovery.
pub struct UdpDiscovery {
    destination: SocketAddr,
    socket: UdpSocket,
}

impl UdpDiscovery {
    /// Binds a socket and configures its broadcast destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket cannot bind or enable broadcast mode.
    pub async fn bind(bind_address: SocketAddr, destination: SocketAddr) -> Result<Self, UdpError> {
        let socket = UdpSocket::bind(bind_address).await?;
        socket.set_broadcast(true)?;
        Ok(Self {
            destination,
            socket,
        })
    }

    /// Sends one bounded discovery datagram.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the datagram is oversized, or
    /// the UDP send fails.
    pub async fn announce(&self, advertisement: &UdpAdvertisement) -> Result<(), UdpError> {
        let payload = bincode::serialize(advertisement)?;
        if payload.len() > MAX_DISCOVERY_DATAGRAM_BYTES {
            return Err(UdpError::DatagramTooLarge);
        }
        self.socket.send_to(&payload, self.destination).await?;
        Ok(())
    }

    /// Receives one discovery datagram and its source address.
    ///
    /// # Errors
    ///
    /// Returns an error if receiving or decoding the datagram fails, or when
    /// the peer uses an incompatible protocol version.
    pub async fn receive(&self) -> Result<(UdpAdvertisement, SocketAddr), UdpError> {
        let mut buffer = [0; MAX_DISCOVERY_DATAGRAM_BYTES];
        let (length, source) = self.socket.recv_from(&mut buffer).await?;
        let advertisement: UdpAdvertisement = bincode::deserialize(&buffer[..length])?;
        if advertisement.protocol_version != PROTOCOL_VERSION {
            return Err(UdpError::UnsupportedVersion);
        }
        Ok((advertisement, source))
    }

    /// Returns the local address selected by the operating system.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket no longer has a local address.
    pub fn local_addr(&self) -> Result<SocketAddr, UdpError> {
        Ok(self.socket.local_addr()?)
    }
}

/// UDP fallback discovery failures.
#[derive(Debug, Error)]
pub enum UdpError {
    #[error("discovery username is invalid")]
    InvalidUsername,
    #[error("discovery datagram exceeds the configured limit")]
    DatagramTooLarge,
    #[error("peer uses an unsupported discovery protocol version")]
    UnsupportedVersion,
    #[error("UDP socket I/O failed")]
    Io(#[from] std::io::Error),
    #[error("discovery datagram could not be encoded or decoded")]
    Serialization(#[from] Box<bincode::ErrorKind>),
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    use zenpaws_shared::PeerId;

    use super::{UdpAdvertisement, UdpDiscovery};

    #[tokio::test]
    async fn sends_and_receives_a_loopback_advertisement() {
        let loopback = Ipv4Addr::LOCALHOST;
        let receiver = UdpDiscovery::bind(
            SocketAddr::V4(SocketAddrV4::new(loopback, 0)),
            SocketAddr::V4(SocketAddrV4::new(loopback, 0)),
        )
        .await
        .expect("receiver binds");
        let sender = UdpDiscovery::bind(
            SocketAddr::V4(SocketAddrV4::new(loopback, 0)),
            receiver.local_addr().expect("receiver address"),
        )
        .await
        .expect("sender binds");
        let advertisement = UdpAdvertisement::new(PeerId::new(), "peer".to_owned(), 5000, vec![1])
            .expect("advertisement is valid");

        sender
            .announce(&advertisement)
            .await
            .expect("datagram sends");
        let (received_advertisement, _) = receiver.receive().await.expect("datagram receives");

        assert_eq!(received_advertisement, advertisement);
    }
}
