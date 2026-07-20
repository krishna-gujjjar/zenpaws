use std::sync::Arc;

use rcgen::generate_simple_self_signed;
use rustls::{
    ClientConfig, RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zenpaws_shared::{CertificatePin, PeerId, PeerTrustStore, TrustStoreError, TrustedPeer};

fn pin_from_certificate(certificate: &CertificateDer<'_>) -> CertificatePin {
    let digest = Sha256::digest(certificate.as_ref());
    let mut pin = [0; 32];
    pin.copy_from_slice(&digest);
    CertificatePin::new(pin)
}

/// A local self-signed certificate and its private key.
pub struct TlsIdentity {
    certificate: CertificateDer<'static>,
    private_key_der: Vec<u8>,
}

impl TlsIdentity {
    /// Generates a self-signed certificate for the supplied stable peer name.
    ///
    /// # Errors
    ///
    /// Returns an error when the certificate or key cannot be generated.
    pub fn generate(server_name: String) -> Result<Self, TlsError> {
        let certified = generate_simple_self_signed(vec![server_name])?;
        let certificate = certified.cert.der().clone();
        Ok(Self {
            certificate,
            private_key_der: certified.signing_key.serialize_der(),
        })
    }

    /// Returns this identity's certificate pin for TOFU persistence.
    #[must_use]
    pub fn pin(&self) -> CertificatePin {
        pin_from_certificate(&self.certificate)
    }

    /// Applies TOFU pinning and builds a client configuration for a peer.
    ///
    /// On first contact the certificate is recorded. On reconnect the
    /// presented certificate must produce the same stored SHA-256 pin.
    ///
    /// # Errors
    ///
    /// Returns an error for changed pins, unavailable persistent storage, or
    /// invalid TLS trust-anchor configuration.
    pub fn trust_peer_certificate<S>(
        trust_store: &S,
        peer_id: PeerId,
        presented_certificate: CertificateDer<'static>,
    ) -> Result<Arc<ClientConfig>, TlsError>
    where
        S: PeerTrustStore + ?Sized,
    {
        let pin = pin_from_certificate(&presented_certificate);
        let certificate = if let Some(existing) = trust_store.trusted_peer(peer_id)? {
            if existing.pin() != pin {
                return Err(TlsError::Trust(TrustStoreError::PinMismatch));
            }
            CertificateDer::from(existing.certificate_der().to_vec())
        } else {
            trust_store.trust_first_use(TrustedPeer::new(
                peer_id,
                presented_certificate.as_ref().to_vec(),
                pin,
            ))?;
            presented_certificate
        };
        Self::client_config_for_peer(certificate)
    }

    /// Builds a TLS server configuration for incoming peer connections.
    ///
    /// # Errors
    ///
    /// Returns an error when the certificate and private key cannot form a
    /// valid TLS server configuration.
    pub fn server_config(&self) -> Result<Arc<ServerConfig>, TlsError> {
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![self.certificate.clone()],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.private_key_der.clone())),
            )?;
        Ok(Arc::new(config))
    }

    /// Restores a persisted self-signed certificate and PKCS#8 private key.
    #[must_use]
    pub fn from_der(certificate_der: Vec<u8>, private_key_der: Vec<u8>) -> Self {
        Self {
            certificate: CertificateDer::from(certificate_der),
            private_key_der,
        }
    }

    /// Returns a clone of this peer's public certificate.
    #[must_use]
    pub fn certificate(&self) -> CertificateDer<'static> {
        self.certificate.clone()
    }

    /// Returns a clone of the PKCS#8 private key for local persistence.
    #[must_use]
    pub fn private_key_der(&self) -> Vec<u8> {
        self.private_key_der.clone()
    }

    /// Builds a client configuration that trusts exactly one peer certificate.
    ///
    /// The caller supplies the certificate stored after an explicit first-use
    /// trust decision. No system root or arbitrary certificate is accepted.
    ///
    /// # Errors
    ///
    /// Returns an error when the pinned certificate cannot be added as a trust
    /// anchor.
    pub fn client_config_for_peer(
        peer_certificate: CertificateDer<'static>,
    ) -> Result<Arc<ClientConfig>, TlsError> {
        let mut roots = RootCertStore::empty();
        roots.add(peer_certificate)?;
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Ok(Arc::new(config))
    }
}

/// Certificate generation and TLS configuration failures.
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("self-signed certificate generation failed")]
    CertificateGeneration(#[from] rcgen::Error),
    #[error("TLS configuration failed")]
    Rustls(#[from] rustls::Error),
    #[error("TOFU trust validation failed")]
    Trust(#[from] TrustStoreError),
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use zenpaws_shared::{PeerId, PeerTrustStore, TrustStoreError, TrustedPeer};

    use super::TlsIdentity;

    struct MemoryTrustStore(Mutex<Option<TrustedPeer>>);

    impl PeerTrustStore for MemoryTrustStore {
        fn trusted_peer(&self, peer_id: PeerId) -> Result<Option<TrustedPeer>, TrustStoreError> {
            Ok(self
                .0
                .lock()
                .expect("test lock is available")
                .as_ref()
                .filter(|peer| peer.peer_id() == peer_id)
                .cloned())
        }

        fn trust_first_use(&self, peer: TrustedPeer) -> Result<(), TrustStoreError> {
            *self.0.lock().expect("test lock is available") = Some(peer);
            Ok(())
        }
    }

    #[test]
    fn generated_identity_creates_tls_configs() {
        let identity = TlsIdentity::generate("zenpaws-test.local".to_owned())
            .expect("certificate generation succeeds");

        assert!(identity.server_config().is_ok());
        assert!(TlsIdentity::client_config_for_peer(identity.certificate()).is_ok());
    }

    #[test]
    fn tofu_rejects_a_changed_certificate() {
        let store = MemoryTrustStore(Mutex::new(None));
        let peer_id = PeerId::new();
        let first = TlsIdentity::generate("first.local".to_owned()).expect("first identity");
        let changed = TlsIdentity::generate("changed.local".to_owned()).expect("changed identity");

        TlsIdentity::trust_peer_certificate(&store, peer_id, first.certificate())
            .expect("first certificate is trusted");
        assert!(
            TlsIdentity::trust_peer_certificate(&store, peer_id, changed.certificate()).is_err()
        );
    }
}
