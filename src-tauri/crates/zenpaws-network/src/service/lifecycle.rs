use std::{net::SocketAddr, sync::Arc, time::Duration};

use rustls::pki_types::{CertificateDer, ServerName};
use zenpaws_shared::{PeerId, PeerTrustStore, TrustStoreError, ZenPawsEvent};

use super::{NetworkService, PeerConnection, ServiceError};
use crate::TlsError;

impl NetworkService {
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
                        self.register_connection(connection);
                    }
                    Err(error) => {
                        super::push_log(&self.logs, &format!("Inbound peer accept failed: {error:?}"));
                        tokio::time::sleep(retry.next_delay()).await;
                    }
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
        super::push_log(&self.logs, "mDNS discovery loop started");
        let Some(discovery) = self.discovery.as_ref() else {
            super::push_log(
                &self.logs,
                "mDNS discovery loop disabled because mDNS is unavailable",
            );
            return;
        };
        loop {
            let peer = tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                    continue;
                }
                result = discovery.next_peer() => match result {
                    Ok(peer) => {
                        super::push_log(&self.logs, &format!("mDNS peer discovered: {} at {}", peer.peer_id.as_uuid(), peer.endpoint));
                        peer
                    }
                    Err(error) => {
                        super::push_log(&self.logs, &format!("mDNS browse error: {error:?}"));
                        break;
                    }
                },
            };
            if peer.peer_id == self.local.peer_id() {
                super::push_log(&self.logs, "Ignored self-discovered mDNS advertisement");
                continue;
            }
            if !self.mark_connecting(peer.peer_id) {
                super::push_log(
                    &self.logs,
                    &format!(
                        "Ignored duplicate peer discovery: {}",
                        peer.peer_id.as_uuid()
                    ),
                );
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
        super::push_log(
            &self.logs,
            "UDP discovery loop started on broadcast port 46235",
        );
        loop {
            if let Err(error) = self.udp.announce(&self.udp_advertisement).await {
                super::push_log(&self.logs, &format!("UDP announcement failed: {error:?}"));
            }
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
                    Err(error) => {
                        super::push_log(&self.logs, &format!("UDP receive failed: {error:?}"));
                        continue;
                    },
                },
                () = tokio::time::sleep(Duration::from_secs(30)) => continue,
            };
            if peer.peer_id == self.local.peer_id() {
                continue;
            }
            if !self.mark_connecting(peer.peer_id) {
                continue;
            }
            super::push_log(
                &self.logs,
                &format!(
                    "UDP peer discovered: {} from {}",
                    peer.peer_id.as_uuid(),
                    peer.endpoint
                ),
            );
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
                    super::push_log(
                        &self.logs,
                        &format!("Peer connected: {}", peer.peer_id.as_uuid()),
                    );
                    self.register_connection(connection);
                    break;
                }
                Err(ServiceError::Tls(TlsError::Trust(TrustStoreError::PinMismatch))) => {
                    self.bus.publish(ZenPawsEvent::PeerTrustViolation {
                        peer_id: peer.peer_id,
                    });
                    super::push_log(
                        &self.logs,
                        &format!("TOFU pin mismatch for peer: {}", peer.peer_id.as_uuid()),
                    );
                    break;
                }
                Err(error) => {
                    super::push_log(
                        &self.logs,
                        &format!("Peer connection failed to {} at {}: {error:?}", peer.peer_id.as_uuid(), peer.endpoint),
                    );
                    tokio::time::sleep(retry.next_delay()).await;
                }
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
        if let Some(discovery) = &self.discovery {
            discovery.shutdown()?;
        }
        Ok(())
    }
}
