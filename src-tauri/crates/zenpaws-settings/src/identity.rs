use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use zenpaws_shared::PeerId;

use crate::SettingsError;

/// Persistent local identity required before the network service starts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalIdentity {
    certificate_der: Vec<u8>,
    peer_id: PeerId,
    private_key_der: Vec<u8>,
    username: String,
}

impl LocalIdentity {
    /// Creates a persistable identity after validating the local display name.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty, oversized, or control-character username.
    pub fn new(
        peer_id: PeerId,
        username: String,
        certificate_der: Vec<u8>,
        private_key_der: Vec<u8>,
    ) -> Result<Self, SettingsError> {
        if username.is_empty() || username.len() > 32 || username.chars().any(char::is_control) {
            return Err(SettingsError::InvalidUsername);
        }
        Ok(Self {
            certificate_der,
            peer_id,
            private_key_der,
            username,
        })
    }

    /// Returns the local peer identity.
    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Returns the local display name.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the self-signed certificate DER.
    #[must_use]
    pub fn certificate_der(&self) -> &[u8] {
        &self.certificate_der
    }

    /// Returns the PKCS#8 private key DER.
    #[must_use]
    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key_der
    }
}

/// Loads a persisted local identity when it exists.
///
/// # Errors
///
/// Returns an error when an existing identity file cannot be read or decoded.
pub fn load_local_identity(path: impl AsRef<Path>) -> Result<Option<LocalIdentity>, SettingsError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

/// Persists a local identity before opening the LAN listener.
///
/// # Errors
///
/// Returns an error when the destination cannot be created or written.
pub fn save_local_identity(
    path: impl Into<PathBuf>,
    identity: &LocalIdentity,
) -> Result<(), SettingsError> {
    let path = path.into();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, serde_json::to_vec(identity)?)?;
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use zenpaws_shared::PeerId;

    use super::{LocalIdentity, load_local_identity, save_local_identity};

    #[test]
    fn persists_identity_material() {
        let peer_id = PeerId::new();
        let path = env::temp_dir().join(format!("zenpaws-identity-{}.json", peer_id.as_uuid()));
        let identity = LocalIdentity::new(peer_id, "owner".to_owned(), vec![1], vec![2])
            .expect("identity is valid");

        save_local_identity(&path, &identity).expect("identity saves");
        let loaded = load_local_identity(&path).expect("identity loads");

        assert_eq!(loaded, Some(identity));
        std::fs::remove_file(path).expect("test file is removable");
    }
}
