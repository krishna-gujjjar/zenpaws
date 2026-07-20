//! Persistent local settings used by the networking foundation.
//!
//! See `docs/12_SECURITY_MODEL.md`.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
mod identity;

pub use identity::{LocalIdentity, load_local_identity, save_local_identity};

use zenpaws_shared::{PeerId, PeerTrustStore, TrustStoreError, TrustedPeer};

/// JSON-backed TOFU certificate storage in the application data directory.
#[derive(Debug)]
pub struct JsonTrustStore {
    path: PathBuf,
    peers: Mutex<HashMap<PeerId, TrustedPeer>>,
}

impl JsonTrustStore {
    /// Opens existing trusted-peer data, or starts with an empty store.
    ///
    /// # Errors
    ///
    /// Returns an error when the existing file cannot be read or decoded.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, SettingsError> {
        let path = path.into();
        let peers = read_peers(&path)?;
        Ok(Self {
            path,
            peers: Mutex::new(peers),
        })
    }
}

impl PeerTrustStore for JsonTrustStore {
    fn trusted_peer(&self, peer_id: PeerId) -> Result<Option<TrustedPeer>, TrustStoreError> {
        let peers = self
            .peers
            .lock()
            .map_err(|_| TrustStoreError::Storage("trust store lock poisoned".to_owned()))?;
        Ok(peers.get(&peer_id).cloned())
    }

    fn trust_first_use(&self, peer: TrustedPeer) -> Result<(), TrustStoreError> {
        let mut peers = self
            .peers
            .lock()
            .map_err(|_| TrustStoreError::Storage("trust store lock poisoned".to_owned()))?;
        if let Some(existing) = peers.get(&peer.peer_id()) {
            if existing.pin() == peer.pin() {
                return Ok(());
            }
            return Err(TrustStoreError::PinMismatch);
        }

        let mut updated = peers.clone();
        updated.insert(peer.peer_id(), peer);
        write_peers(&self.path, &updated)
            .map_err(|error| TrustStoreError::Storage(error.to_string()))?;
        *peers = updated;
        drop(peers);
        Ok(())
    }
}

fn read_peers(path: &Path) -> Result<HashMap<PeerId, TrustedPeer>, SettingsError> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice::<TrustFile>(&bytes)?.peers)
}

fn write_peers(path: &Path, peers: &HashMap<PeerId, TrustedPeer>) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec(&TrustFile {
        peers: peers.clone(),
    })?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct TrustFile {
    peers: HashMap<PeerId, TrustedPeer>,
}

/// JSON trust-store failures.
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings file I/O failed")]
    Io(#[from] std::io::Error),
    #[error("settings file is invalid")]
    Json(#[from] serde_json::Error),
    #[error("local username is invalid")]
    InvalidUsername,
}

#[cfg(test)]
mod tests {
    use std::env;

    use zenpaws_shared::{CertificatePin, PeerTrustStore, TrustedPeer};

    use super::JsonTrustStore;

    #[test]
    fn rejects_a_changed_certificate_pin() {
        let peer = zenpaws_shared::PeerId::new();
        let path = env::temp_dir().join(format!("zenpaws-trust-{}.json", peer.as_uuid()));
        let store = JsonTrustStore::open(&path).expect("store opens");
        let first = TrustedPeer::new(peer, vec![1], CertificatePin::new([1; 32]));
        let changed = TrustedPeer::new(peer, vec![2], CertificatePin::new([2; 32]));

        store
            .trust_first_use(first)
            .expect("first contact is trusted");
        assert!(store.trust_first_use(changed).is_err());

        std::fs::remove_file(path).expect("test file is removable");
    }
}
