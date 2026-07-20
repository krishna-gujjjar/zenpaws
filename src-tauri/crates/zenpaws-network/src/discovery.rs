use base64::{Engine, engine::general_purpose::STANDARD};
use mdns_sd::{Receiver, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::net::{SocketAddr, SocketAddrV4};
use thiserror::Error;
use zenpaws_shared::PeerId;

/// DNS-SD service type used by `ZenPaws` peers.
pub const SERVICE_TYPE: &str = "_zenpaws._tcp.local.";

/// A resolved peer that is ready for pinned outbound TLS.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredPeer {
    /// Certificate acquired from the peer's mDNS TXT record.
    pub certificate_der: Vec<u8>,
    /// Resolved IPv4 TCP endpoint.
    pub endpoint: SocketAddr,
    /// Stable peer identity.
    pub peer_id: PeerId,
    /// Display name.
    pub username: String,
}

/// mDNS advertisement and browse service for the local LAN.
pub struct MdnsDiscovery {
    daemon: ServiceDaemon,
    events: Receiver<ServiceEvent>,
}

impl MdnsDiscovery {
    /// Starts a local DNS-SD daemon and browses for `ZenPaws` peers.
    ///
    /// # Errors
    ///
    /// Returns an error if the system cannot create the mDNS daemon or start
    /// browsing the `ZenPaws` service type.
    pub fn start() -> Result<Self, DiscoveryError> {
        let daemon = ServiceDaemon::new()?;
        let events = daemon.browse(SERVICE_TYPE)?;
        Ok(Self { daemon, events })
    }

    /// Announces this peer's TCP listener through mDNS.
    ///
    /// # Errors
    ///
    /// Returns an error if the service record is invalid or cannot be
    /// registered with the local mDNS daemon.
    pub fn advertise(
        &self,
        peer_id: PeerId,
        username: &str,
        port: u16,
        certificate_der: &[u8],
    ) -> Result<(), DiscoveryError> {
        let mut properties = std::collections::HashMap::from([
            ("uuid".to_owned(), peer_id.as_uuid().to_string()),
            ("username".to_owned(), username.to_owned()),
        ]);
        let certificate = STANDARD.encode(certificate_der);
        let chunks = certificate.as_bytes().chunks(200).collect::<Vec<_>>();
        properties.insert("certificate_parts".to_owned(), chunks.len().to_string());
        for (index, chunk) in chunks.into_iter().enumerate() {
            let value = std::str::from_utf8(chunk)
                .map_err(|_| DiscoveryError::InvalidCertificateEncoding)?;
            properties.insert(format!("certificate_{index}"), value.to_owned());
        }
        let instance_name = format!("zenpaws-{}", peer_id.as_uuid());
        let service = ServiceInfo::new(SERVICE_TYPE, &instance_name, "", "", port, properties)?;
        self.daemon.register(service)?;
        Ok(())
    }

    /// Waits for the next DNS-SD event produced by the local daemon.
    ///
    /// # Errors
    ///
    /// Returns an error when the mDNS daemon's event channel closes.
    pub async fn next_event(&self) -> Result<ServiceEvent, DiscoveryError> {
        self.events
            .recv_async()
            .await
            .map_err(|_| DiscoveryError::EventChannelClosed)
    }

    /// Waits for a resolved peer carrying the required TLS metadata.
    ///
    /// # Errors
    ///
    /// Returns an error for a closed event channel or malformed peer metadata.
    pub async fn next_peer(&self) -> Result<DiscoveredPeer, DiscoveryError> {
        loop {
            if let ServiceEvent::ServiceResolved(service) = self.next_event().await? {
                return parse_resolved_peer(&service);
            }
        }
    }

    /// Stops advertising and browsing before application shutdown.
    ///
    /// # Errors
    ///
    /// Returns an error if the mDNS daemon cannot be shut down cleanly.
    pub fn shutdown(&self) -> Result<(), DiscoveryError> {
        self.daemon.shutdown()?;
        Ok(())
    }
}

fn parse_resolved_peer(
    service: &mdns_sd::ResolvedService,
) -> Result<DiscoveredPeer, DiscoveryError> {
    let peer_id = service
        .get_property_val_str("uuid")
        .ok_or(DiscoveryError::MissingMetadata)?
        .parse()
        .map(PeerId::from_uuid)
        .map_err(|_| DiscoveryError::InvalidMetadata)?;
    let username = service
        .get_property_val_str("username")
        .ok_or(DiscoveryError::MissingMetadata)?
        .to_owned();
    let count = service
        .get_property_val_str("certificate_parts")
        .ok_or(DiscoveryError::MissingMetadata)?
        .parse::<usize>()
        .map_err(|_| DiscoveryError::InvalidMetadata)?;
    let mut encoded = String::new();
    for index in 0..count {
        encoded.push_str(
            service
                .get_property_val_str(&format!("certificate_{index}"))
                .ok_or(DiscoveryError::MissingMetadata)?,
        );
    }
    let certificate_der = STANDARD
        .decode(encoded)
        .map_err(|_| DiscoveryError::InvalidCertificateEncoding)?;
    let address = service
        .get_addresses_v4()
        .into_iter()
        .next()
        .ok_or(DiscoveryError::MissingIpv4Address)?;
    Ok(DiscoveredPeer {
        certificate_der,
        endpoint: SocketAddr::V4(SocketAddrV4::new(address, service.get_port())),
        peer_id,
        username,
    })
}

/// mDNS discovery failures.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("mDNS operation failed")]
    Mdns(#[from] mdns_sd::Error),
    #[error("mDNS event channel closed")]
    EventChannelClosed,
    #[error("certificate encoding was invalid")]
    InvalidCertificateEncoding,
    #[error("peer mDNS metadata was invalid")]
    InvalidMetadata,
    #[error("peer mDNS metadata was incomplete")]
    MissingMetadata,
    #[error("peer has no resolved IPv4 address")]
    MissingIpv4Address,
}
