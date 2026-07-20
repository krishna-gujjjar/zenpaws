use std::sync::{Arc, Mutex};

use rusqlite::params;
use zenpaws_shared::{CertificatePin, PeerId, PeerTrustStore, TrustStoreError, TrustedPeer};

use crate::Database;

/// TOFU trust storage backed by the local `SQLite` database.
#[derive(Clone)]
pub struct DatabaseTrustStore {
    database: Arc<Mutex<Database>>,
}

impl DatabaseTrustStore {
    /// Wraps Tauri-managed database state for network trust operations.
    #[must_use]
    pub const fn new(database: Arc<Mutex<Database>>) -> Self {
        Self { database }
    }
}

impl PeerTrustStore for DatabaseTrustStore {
    fn trusted_peer(&self, peer_id: PeerId) -> Result<Option<TrustedPeer>, TrustStoreError> {
        let result = self
            .database
            .lock()
            .map_err(|_| TrustStoreError::Storage("database lock poisoned".to_owned()))?
            .connection
            .query_row(
                "SELECT p.pubkey_fingerprint, c.certificate_der
             FROM peers AS p
             JOIN peer_certificates AS c ON c.peer_uuid = p.uuid
             WHERE p.uuid = ?1",
                [peer_id.as_uuid().to_string()],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            );
        match result {
            Ok((fingerprint, certificate_der)) => {
                let pin: [u8; 32] = fingerprint.try_into().map_err(|_| {
                    TrustStoreError::Storage("stored fingerprint has invalid length".to_owned())
                })?;
                Ok(Some(TrustedPeer::new(
                    peer_id,
                    certificate_der,
                    CertificatePin::new(pin),
                )))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(TrustStoreError::Storage(error.to_string())),
        }
    }

    fn trust_first_use(&self, peer: TrustedPeer) -> Result<(), TrustStoreError> {
        if let Some(existing) = self.trusted_peer(peer.peer_id())? {
            return if existing.pin() == peer.pin() {
                Ok(())
            } else {
                Err(TrustStoreError::PinMismatch)
            };
        }
        let database = self
            .database
            .lock()
            .map_err(|_| TrustStoreError::Storage("database lock poisoned".to_owned()))?;
        let transaction = database
            .connection
            .unchecked_transaction()
            .map_err(|error| TrustStoreError::Storage(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO peers (uuid, username, pubkey_fingerprint, last_seen_at)
             VALUES (?1, 'Unknown', ?2, unixepoch())
             ON CONFLICT(uuid) DO UPDATE SET last_seen_at = unixepoch()",
                params![
                    peer.peer_id().as_uuid().to_string(),
                    peer.pin().as_bytes().to_vec()
                ],
            )
            .map_err(|error| TrustStoreError::Storage(error.to_string()))?;
        transaction
            .execute(
                "INSERT INTO peer_certificates (peer_uuid, certificate_der)
             VALUES (?1, ?2)
             ON CONFLICT(peer_uuid) DO NOTHING",
                params![peer.peer_id().as_uuid().to_string(), peer.certificate_der()],
            )
            .map_err(|error| TrustStoreError::Storage(error.to_string()))?;
        transaction
            .commit()
            .map_err(|error| TrustStoreError::Storage(error.to_string()))?;
        drop(database);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use zenpaws_shared::{CertificatePin, PeerTrustStore, TrustedPeer};

    use super::{Database, DatabaseTrustStore, PeerId};

    #[test]
    fn persists_first_use_and_rejects_changed_pin() {
        let database = Arc::new(Mutex::new(
            Database::open_in_memory().expect("database opens"),
        ));
        let trust = DatabaseTrustStore::new(database);
        let peer_id = PeerId::new();
        let first = TrustedPeer::new(peer_id, vec![1], CertificatePin::new([1; 32]));
        let changed = TrustedPeer::new(peer_id, vec![2], CertificatePin::new([2; 32]));

        trust
            .trust_first_use(first.clone())
            .expect("first trust succeeds");
        assert_eq!(
            trust.trusted_peer(peer_id).expect("peer reads"),
            Some(first)
        );
        assert!(trust.trust_first_use(changed).is_err());
    }
}
