//! Data-driven pet manifests, local state, and synchronized-state rate limits.
//!
//! See `docs/10_PET_ENGINE_SPECIFICATION.md`.

mod engine;
mod manifest;
mod registry;
mod state;

pub use engine::{PetEngine, PetEngineError};
pub use manifest::{PetManifest, PetManifestError};
pub use registry::{PetRegistry, PetRegistryError};
pub use state::{PetInstance, PetState, PetStateError};
