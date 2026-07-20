use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use rustls::pki_types::{CertificateDer, ServerName};
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsStream;
use zenpaws_shared::{EventBus, PeerId, PeerTrustStore, TrustStoreError, ZenPawsEvent};

use crate::{
    ConnectionStateMachine, Envelope, FrameError, HandshakeError, HandshakeIdentity, MdnsDiscovery,
    TlsError, TlsIdentity, TransportError, UDP_DISCOVERY_PORT, UdpAdvertisement, UdpDiscovery,
    accept_tls, client_handshake, connect_tls, read_envelope, server_handshake, write_envelope,
};

/// Listener, mDNS advertisement, and inbound encrypted peer lifecycle.
pub struct NetworkService {
    connecting_peers: Mutex<HashSet<PeerId>>,
    discovery: MdnsDiscovery,
    udp: UdpDiscovery,
    udp_advertisement: UdpAdvertisement,
    heartbeat_timeout: Duration,
    local: HandshakeIdentity,
    listener: TcpListener,
    server_config: Arc<rustls::ServerConfig>,
    bus: EventBus,
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
        let discovery = MdnsDiscovery::start()?;
        let tcp_port = listener.local_addr()?.port();
        let certificate = identity.certificate();
        discovery.advertise(peer_id, local.username(), tcp_port, certificate.as_ref())?;
        let udp = UdpDiscovery::bind(
            SocketAddr::from(([0, 0, 0, 0], UDP_DISCOVERY_PORT)),
            SocketAddr::from(([255, 255, 255, 255], UDP_DISCOVERY_PORT)),
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

    /// Runs the inbound accept loop with bounded recovery after listener errors.
    pub async fn run_inbound(self: Arc<Self>, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let mut retry =
            crate::RetryBackoff::new(Duration::from_millis(250), Duration::from_secs(5));
        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                result = self.accept_peer() => match result {
                    Ok(connection) => {
                        retry.reset();
                        tokio::spawn(async move { connection.monitor().await });
                    }
                    Err(_) => tokio::time::sleep(retry.next_delay()).await,
                },
            }
        }
    }

    /// Connects to one resolved mDNS peer through the full TOFU lifecycle.
    ///
    /// # Errors
    ///
    /// Returns an error for self-discovery, invalid TLS naming, trust failure,
    /// transport failure, or application-handshake failure.
    pub async fn connect_discovered_peer<S>(
        &self,
        peer: crate::DiscoveredPeer,
        trust_store: &S,
    ) -> Result<PeerConnection, ServiceError>
    where
        S: PeerTrustStore + ?Sized,
    {
        if peer.peer_id == self.local.peer_id() {
            return Err(ServiceError::SelfConnection);
        }
        let name = ServerName::try_from(format!("zenpaws-{}.local", peer.peer_id.as_uuid()))
            .map_err(|_| ServiceError::InvalidServerName)?
            .to_owned();
        self.connect_peer(
            peer.endpoint,
            name,
            peer.peer_id,
            CertificateDer::from(peer.certificate_der),
            trust_store,
        )
        .await
    }

    /// Reacts to resolved mDNS peers without polling or port scanning.
    pub async fn run_discovery(
        self: Arc<Self>,
        trust_store: Arc<dyn PeerTrustStore>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            let peer = tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                    continue;
                }
                result = self.discovery.next_peer() => match result {
                    Ok(peer) => peer,
                    Err(_) => break,
                },
            };
            if peer.peer_id == self.local.peer_id() || !self.mark_connecting(peer.peer_id) {
                continue;
            }
            let service = Arc::clone(&self);
            let trust_store = Arc::clone(&trust_store);
            tokio::spawn(async move { service.connect_with_retry(peer, trust_store).await });
        }
    }

    /// Announces and receives the UDP discovery fallback at a low fixed rate.
    pub async fn run_udp(
        self: Arc<Self>,
        trust_store: Arc<dyn PeerTrustStore>,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            let _ = self.udp.announce(&self.udp_advertisement).await;
            let peer = tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                    continue;
                }
                received = self.udp.receive() => match received {
                    Ok((advertisement, source)) => crate::DiscoveredPeer {
                        certificate_der: advertisement.certificate_der,
                        endpoint: SocketAddr::new(source.ip(), advertisement.tcp_port),
                        peer_id: advertisement.peer_id,
                        username: advertisement.username,
                    },
                    Err(_) => continue,
                },
                () = tokio::time::sleep(Duration::from_secs(30)) => continue,
            };
            if peer.peer_id == self.local.peer_id() || !self.mark_connecting(peer.peer_id) {
                continue;
            }
            let service = Arc::clone(&self);
            let trust_store = Arc::clone(&trust_store);
            tokio::spawn(async move { service.connect_with_retry(peer, trust_store).await });
        }
    }

    async fn connect_with_retry(
        self: Arc<Self>,

        peer: crate::DiscoveredPeer,
        trust_store: Arc<dyn PeerTrustStore>,
    ) {
        let mut retry =
            crate::RetryBackoff::new(Duration::from_millis(250), Duration::from_secs(5));
        loop {
            match self
                .connect_discovered_peer(peer.clone(), trust_store.as_ref())
                .await
            {
                Ok(connection) => {
                    connection.monitor().await;
                    break;
                }
                Err(ServiceError::Tls(TlsError::Trust(TrustStoreError::PinMismatch))) => {
                    self.bus.publish(ZenPawsEvent::PeerTrustViolation {
                        peer_id: peer.peer_id,
                    });
                    break;
                }
                Err(_) => tokio::time::sleep(retry.next_delay()).await,
            }
        }
        self.clear_connecting(peer.peer_id);
    }

    fn mark_connecting(&self, peer_id: PeerId) -> bool {
        self.connecting_peers
            .lock()
            .is_ok_and(|mut peers| peers.insert(peer_id))
    }

    fn clear_connecting(&self, peer_id: PeerId) {
        if let Ok(mut peers) = self.connecting_peers.lock() {
            peers.remove(&peer_id);
        }
    }

    /// Stops mDNS activity before the application exits.
    ///
    /// # Errors
    ///
    /// Returns an error when the mDNS daemon cannot shut down cleanly.
    pub fn shutdown(&self) -> Result<(), ServiceError> {
        self.discovery.shutdown()?;
        Ok(())
    }
}

/// One accepted, TLS-protected peer connection.
pub struct PeerConnection {
    address: SocketAddr,
    heartbeat_timeout: Duration,
    remote: HandshakeIdentity,
    state: ConnectionStateMachine,
    stream: TlsStream<TcpStream>,
}

impl PeerConnection {
    /// Returns the remote TCP endpoint.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns the identity validated by the application handshake.
    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.remote.peer_id()
    }

    /// Sends a heartbeat without allocating a polling task.
    ///
    /// # Errors
    ///
    /// Returns an error when encrypted framing fails.
    pub async fn send_heartbeat(&mut self) -> Result<(), ServiceError> {
        write_envelope(&mut self.stream, &Envelope::Heartbeat).await?;
        Ok(())
    }

    /// Receives one control envelope and refreshes heartbeat state when needed.
    ///
    /// # Errors
    ///
    /// Returns an error when encrypted framing fails or the peer state is invalid.
    pub async fn receive(&mut self) -> Result<Envelope, ServiceError> {
        let envelope = read_envelope(&mut self.stream).await?;
        if matches!(envelope, Envelope::Heartbeat) {
            self.state.heartbeat(self.heartbeat_timeout)?;
        }
        Ok(envelope)
    }

    /// Monitors inbound control frames until the peer disconnects or a frame fails.
    async fn monitor(mut self) {
        while self.receive().await.is_ok() {}
        let _ = self.state.disconnected();
    }

    /// Applies the configured heartbeat deadline.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection state cannot transition.
    pub fn check_heartbeat(&mut self, now: Instant) -> Result<(), ServiceError> {
        self.state.check_heartbeat(now)?;
        Ok(())
    }
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
}
