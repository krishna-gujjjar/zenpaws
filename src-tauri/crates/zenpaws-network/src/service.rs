mod connection;
mod error;
mod lifecycle;

pub use connection::PeerConnection;
pub use error::ServiceError;

use std::{
    collections::{HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};

use rustls::pki_types::{CertificateDer, ServerName};
use tokio::{net::TcpListener, sync::mpsc};
use zenpaws_shared::{EventBus, PeerId, PeerTrustStore};

use crate::{
    ConnectionStateMachine, Envelope, HandshakeIdentity, MdnsDiscovery, TlsIdentity,
    UDP_DISCOVERY_PORT, UdpAdvertisement, UdpDiscovery, accept_tls, client_handshake, connect_tls,
    server_handshake,
};

/// Runtime discovery and connection diagnostics.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnostics {
    pub broadcast_address: String,
    pub connected_peers: usize,
    pub local_address: Option<String>,
    pub mdns_available: bool,
    pub recent_logs: Vec<String>,
    pub tcp_address: String,
    pub udp_address: String,
}

/// Listener, mDNS advertisement, and inbound encrypted peer lifecycle.
pub struct NetworkService {
    connecting_peers: Mutex<HashSet<PeerId>>,
    discovery: Option<MdnsDiscovery>,
    udp: UdpDiscovery,
    udp_advertisement: UdpAdvertisement,
    heartbeat_timeout: Duration,
    local: HandshakeIdentity,
    listener: TcpListener,
    server_config: Arc<rustls::ServerConfig>,
    bus: EventBus,
    peers: Mutex<HashMap<PeerId, mpsc::Sender<Envelope>>>,
    logs: Arc<Mutex<Vec<String>>>,
}

impl NetworkService {
    /// Binds the TCP listener and advertises it through mDNS.
    ///
    /// # Errors
    ///
    /// Returns an error when identity validation, TCP binding, TLS setup, or
    /// mDNS advertising fails.
    pub async fn bind(
        bind_address: SocketAddr,
        peer_id: PeerId,
        username: String,
        identity: &TlsIdentity,
        bus: EventBus,
        heartbeat_timeout: Duration,
    ) -> Result<Self, ServiceError> {
        let local = HandshakeIdentity::new(peer_id, username)?;
        let listener = TcpListener::bind(bind_address).await?;
        let tcp_port = listener.local_addr()?.port();
        let logs = Arc::new(Mutex::new(vec![format!(
            "TCP listener bound on {}",
            listener.local_addr()?
        )]));
        let certificate = identity.certificate();
        let discovery = match MdnsDiscovery::start() {
            Ok(discovery) => {
                match discovery.advertise(peer_id, local.username(), tcp_port, certificate.as_ref())
                {
                    Ok(()) => {
                        push_log(&logs, "mDNS advertisement started");
                        Some(discovery)
                    }
                    Err(error) => {
                        push_log(&logs, &format!("mDNS advertisement failed: {error:?}"));
                        None
                    }
                }
            }
            Err(error) => {
                push_log(&logs, &format!("mDNS startup failed: {error:?}"));
                None
            }
        };
        let udp = UdpDiscovery::bind(
            SocketAddr::from(([0, 0, 0, 0], UDP_DISCOVERY_PORT)),
            lan_broadcast_address(UDP_DISCOVERY_PORT),
        )
        .await?;
        let udp_advertisement = UdpAdvertisement::new(
            peer_id,
            local.username().to_owned(),
            tcp_port,
            certificate.as_ref().to_vec(),
        )?;
        Ok(Self {
            connecting_peers: Mutex::new(HashSet::new()),
            discovery,
            udp,
            udp_advertisement,
            heartbeat_timeout,
            local,
            listener,
            server_config: identity.server_config()?,
            bus,
            peers: Mutex::new(HashMap::new()),
            logs,
        })
    }

    /// Returns the local stable peer identity advertised by this service.
    #[must_use]
    pub const fn local_peer_id(&self) -> PeerId {
        self.local.peer_id()
    }

    /// Returns the TCP listener address assigned to this service.
    ///
    /// # Errors
    ///
    /// Returns an error when the listener no longer has a local address.
    pub fn local_addr(&self) -> Result<SocketAddr, ServiceError> {
        Ok(self.listener.local_addr()?)
    }

    /// Returns the current LAN discovery and connection state.
    ///
    /// # Errors
    ///
    /// Returns an error when the UDP socket or peer registry cannot be queried.
    pub fn diagnostics(&self) -> Result<NetworkDiagnostics, ServiceError> {
        let connected_peers = self
            .peers
            .lock()
            .map_err(|_| ServiceError::PeerRegistryPoisoned)?
            .len();
        let local_address = local_lan_address();
        let broadcast_address = lan_broadcast_address(UDP_DISCOVERY_PORT).to_string();
        let recent_logs = self
            .logs
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default();
        Ok(NetworkDiagnostics {
            broadcast_address,
            connected_peers,
            local_address,
            mdns_available: self.discovery.is_some(),
            recent_logs,
            tcp_address: self.local_addr()?.to_string(),
            udp_address: self.udp.local_addr()?.to_string(),
        })
    }

    /// Queues one envelope for every currently connected peer.
    pub fn broadcast(&self, envelope: &Envelope) {
        if let Ok(mut peers) = self.peers.lock() {
            peers.retain(|_, sender| sender.try_send(envelope.clone()).is_ok());
        }
    }

    /// Queues one envelope for a specific connected peer.
    pub fn send_to(&self, peer_id: PeerId, envelope: &Envelope) -> Result<(), ServiceError> {
        let peers = self
            .peers
            .lock()
            .map_err(|_| ServiceError::PeerRegistryPoisoned)?;
        let sender = peers.get(&peer_id).ok_or(ServiceError::PeerNotConnected)?;
        sender
            .try_send(envelope.clone())
            .map_err(|_| ServiceError::PeerChannelClosed)
    }

    fn register_connection(&self, connection: PeerConnection) {
        let peer_id = connection.peer_id();
        let sender = connection.start();
        let _ = sender.try_send(Envelope::SyncRequest {
            room: "shared".to_owned(),
            since: crate::LamportClock {
                counter: 0,
                peer_id: self.local.peer_id(),
            },
        });
        if let Ok(mut peers) = self.peers.lock() {
            peers.insert(peer_id, sender);
        }
    }

    /// Accepts TLS and completes the server half of the application handshake.
    ///
    /// # Errors
    ///
    /// Returns an error when TCP, TLS, framing, or handshake validation fails.
    pub async fn accept_peer(&self) -> Result<PeerConnection, ServiceError> {
        let (mut stream, address) =
            accept_tls(&self.listener, Arc::clone(&self.server_config)).await?;
        let remote = server_handshake(&mut stream, &self.local).await?;
        let mut state = ConnectionStateMachine::new(remote.peer_id(), self.bus.clone());
        state.connected(self.heartbeat_timeout)?;
        Ok(PeerConnection {
            address,
            heartbeat_timeout: self.heartbeat_timeout,
            remote,
            state,
            stream,
        })
    }

    /// Connects to a discovered peer using TOFU-pinned TLS and application identity validation.
    ///
    /// # Errors
    ///
    /// Returns an error when pin validation, TCP/TLS connection, framing, or
    /// remote identity validation fails.
    pub async fn connect_peer<S>(
        &self,
        address: SocketAddr,
        server_name: ServerName<'static>,
        expected_peer: PeerId,
        presented_certificate: CertificateDer<'static>,
        trust_store: &S,
    ) -> Result<PeerConnection, ServiceError>
    where
        S: PeerTrustStore + ?Sized,
    {
        let client_config =
            TlsIdentity::trust_peer_certificate(trust_store, expected_peer, presented_certificate)?;
        let mut stream = connect_tls(address, server_name, client_config).await?;
        let remote = client_handshake(&mut stream, &self.local, expected_peer).await?;
        let mut state = ConnectionStateMachine::new(remote.peer_id(), self.bus.clone());
        state.connected(self.heartbeat_timeout)?;
        Ok(PeerConnection {
            address,
            heartbeat_timeout: self.heartbeat_timeout,
            remote,
            state,
            stream,
        })
    }
}

pub(super) fn push_log(logs: &Arc<Mutex<Vec<String>>>, message: &str) {
    if let Ok(mut logs) = logs.lock() {
        logs.push(message.to_owned());
        if logs.len() > 100 {
            logs.remove(0);
        }
    }
}

fn lan_broadcast_address(port: u16) -> SocketAddr {
    let broadcast = local_lan_address()
        .and_then(|address| address.parse::<Ipv4Addr>().ok())
        .map(|address| {
            let octets = address.octets();
            Ipv4Addr::new(octets[0], octets[1], octets[2], 255)
        })
        .unwrap_or(Ipv4Addr::new(255, 255, 255, 255));
    SocketAddr::from((broadcast, port))
}

fn local_lan_address() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::NetworkService;

    #[test]
    fn service_can_be_shared_with_background_tasks() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NetworkService>();
    }
}
