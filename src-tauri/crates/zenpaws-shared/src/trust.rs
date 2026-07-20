use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PeerId;

/// SHA-256 digest of a peer certificate's DER encoding.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CertificatePin([u8; 32]);

impl CertificatePin {
    /// Creates a pin from a SHA-256 digest.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the raw SHA-256 digest.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

/// A certificate trusted for a stable peer identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrustedPeer {
    certificate_der: Vec<u8>,
    peer_id: PeerId,
    pin: CertificatePin,
}

impl TrustedPeer {
    /// Creates a trusted-peer record from verified certificate material.
    #[must_use]
    pub const fn new(peer_id: PeerId, certificate_der: Vec<u8>, pin: CertificatePin) -> Self {
        Self {
            certificate_der,
            peer_id,
            pin,
        }
    }

    /// Returns the peer identity bound to this certificate.
    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Returns the stored certificate DER.
    #[must_use]
    pub fn certificate_der(&self) -> &[u8] {
        &self.certificate_der
    }

    /// Returns the stored certificate pin.
    #[must_use]
    pub const fn pin(&self) -> CertificatePin {
        self.pin
    }
}

/// Persistent TOFU pin storage used by the networking layer.
pub trait PeerTrustStore: Send + Sync {
    /// Returns the certificate previously trusted for a peer.
    ///
    /// # Errors
    ///
    /// Returns an error when persistent trust data cannot be read.
    fn trusted_peer(&self, peer_id: PeerId) -> Result<Option<TrustedPeer>, TrustStoreError>;

    /// Persists a first-use decision or rejects an unexpected pin change.
    ///
    /// # Errors
    ///
    /// Returns an error for a changed pin or failed persistent storage.
    fn trust_first_use(&self, peer: TrustedPeer) -> Result<(), TrustStoreError>;
}

/// TOFU pin-storage failures.
#[derive(Debug, Error)]
pub enum TrustStoreError {
    #[error("peer certificate pin changed")]
    PinMismatch,
    #[error("trust storage failed: {0}")]
    Storage(String),
}
