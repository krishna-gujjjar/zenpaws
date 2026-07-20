use std::{collections::BTreeMap, fs, path::Path};

use thiserror::Error;

use crate::{PetManifest, PetManifestError};

/// Loaded data-driven pet packages keyed by immutable manifest ID.
#[derive(Debug, Default)]
pub struct PetRegistry {
    manifests: BTreeMap<String, PetManifest>,
}

impl PetRegistry {
    /// Loads every valid pet directory directly under the package root.
    ///
    /// # Errors
    ///
    /// Returns an error for unreadable directories, invalid manifests, or
    /// duplicate manifest IDs.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, PetRegistryError> {
        let mut manifests = BTreeMap::new();
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let manifest = PetManifest::load(&path)?;
            if manifests.insert(manifest.id.clone(), manifest).is_some() {
                return Err(PetRegistryError::DuplicateId);
            }
        }
        Ok(Self { manifests })
    }

    /// Returns one installed manifest by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&PetManifest> {
        self.manifests.get(id)
    }

    /// Returns the number of installed pet packages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// Returns whether no pet packages are installed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
}

/// Registry loading failures.
#[derive(Debug, Error)]
pub enum PetRegistryError {
    #[error("pet registry I/O failed")]
    Io(#[from] std::io::Error),
    #[error("pet manifest was invalid")]
    Manifest(#[from] PetManifestError),
    #[error("multiple pet packages use the same manifest ID")]
    DuplicateId,
}
