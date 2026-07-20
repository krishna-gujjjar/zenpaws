use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

/// Data-driven description of one installable desktop pet.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetManifest {
    pub default_scale: f32,
    pub display_name: String,
    pub id: String,
    pub sounds: std::collections::BTreeMap<String, String>,
    pub states: std::collections::BTreeMap<String, SpriteState>,
}

/// Sprite-sheet metadata for one visible pet state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpriteState {
    pub fps: u8,
    pub frame_count: u16,
    pub sprite: String,
}

impl PetManifest {
    /// Loads and validates a manifest and its required license file.
    ///
    /// # Errors
    ///
    /// Returns an error for unreadable, invalid, or unsafe pet assets.
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, PetManifestError> {
        let directory = directory.as_ref();
        if !directory.join("LICENSE.txt").is_file() {
            return Err(PetManifestError::MissingLicense);
        }
        let manifest: Self = serde_json::from_slice(&fs::read(directory.join("manifest.json"))?)?;
        manifest.validate(directory)?;
        Ok(manifest)
    }

    fn validate(&self, directory: &Path) -> Result<(), PetManifestError> {
        if self.id.is_empty()
            || !self.id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
            || self.display_name.is_empty()
            || self.default_scale <= 0.0
        {
            return Err(PetManifestError::InvalidMetadata);
        }
        for state in self.states.values() {
            let sprite = Path::new(&state.sprite);
            if state.frame_count == 0
                || state.fps == 0
                || sprite.is_absolute()
                || sprite
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
                || !asset_path(directory, sprite).is_file()
            {
                return Err(PetManifestError::InvalidSprite);
            }
        }
        Ok(())
    }
}

fn asset_path(directory: &Path, relative: &Path) -> PathBuf {
    directory.join(relative)
}

/// Manifest loading and validation failures.
#[derive(Debug, Error)]
pub enum PetManifestError {
    #[error("pet manifest I/O failed")]
    Io(#[from] std::io::Error),
    #[error("pet manifest JSON was invalid")]
    Json(#[from] serde_json::Error),
    #[error("pet manifest metadata was invalid")]
    InvalidMetadata,
    #[error("pet sprite metadata or file was invalid")]
    InvalidSprite,
    #[error("pet package is missing LICENSE.txt")]
    MissingLicense,
}
