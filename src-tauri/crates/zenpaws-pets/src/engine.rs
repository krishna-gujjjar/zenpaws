use std::{collections::BTreeMap, time::Instant};

use thiserror::Error;
use uuid::Uuid;
use zenpaws_shared::{EventBus, ZenPawsEvent};

use crate::{PetInstance, PetRegistry, PetState, PetStateError};

/// Local pet runtime independent of chat and networking services.
pub struct PetEngine {
    bus: EventBus,
    instances: BTreeMap<Uuid, PetInstance>,
    registry: PetRegistry,
}

impl PetEngine {
    /// Creates an engine from validated installable pet packages.
    #[must_use]
    pub const fn new(registry: PetRegistry, bus: EventBus) -> Self {
        Self {
            bus,
            instances: BTreeMap::new(),
            registry,
        }
    }

    /// Creates one local instance for an installed manifest ID.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested package is not installed.
    pub fn create_instance(&mut self, manifest_id: &str) -> Result<Uuid, PetEngineError> {
        if self.registry.get(manifest_id).is_none() {
            return Err(PetEngineError::UnknownManifest);
        }
        let instance = PetInstance::new();
        let id = instance.id();
        self.instances.insert(id, instance);
        Ok(id)
    }

    /// Updates a local instance and publishes only a permitted discrete event.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown instance or rate-limited state change.
    pub fn set_state(
        &mut self,
        instance_id: Uuid,
        state: PetState,
        now: Instant,
    ) -> Result<(), PetEngineError> {
        let instance = self
            .instances
            .get_mut(&instance_id)
            .ok_or(PetEngineError::UnknownInstance)?;
        if instance.set_state(state, now)?
            && let Some(sync_state) = state.sync_state()
        {
            self.bus.publish(ZenPawsEvent::PetStateChanged {
                pet_instance: instance_id,
                state: sync_state,
            });
        }
        Ok(())
    }
}

/// Pet runtime failures.
#[derive(Debug, Error)]
pub enum PetEngineError {
    #[error("pet manifest is not installed")]
    UnknownManifest,
    #[error("pet instance does not exist")]
    UnknownInstance,
    #[error("pet state update failed")]
    State(#[from] PetStateError),
}
