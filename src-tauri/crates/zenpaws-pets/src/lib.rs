//! Data-driven pet manifests, local state, and synchronized-state rate limits.
//!
//! See `docs/10_PET_ENGINE_SPECIFICATION.md`.

mod behavior;
mod engine;
mod manifest;
mod registry;
mod state;

pub use behavior::{BehaviorDecision, CursorBehavior, PerformanceMode, Point};
pub use engine::{PetEngine, PetEngineError};
pub use manifest::{PetManifest, PetManifestError};
pub use registry::{PetRegistry, PetRegistryError};
pub use state::{PetInstance, PetState, PetStateError};
