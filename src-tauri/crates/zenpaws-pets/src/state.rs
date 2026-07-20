use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use zenpaws_shared::PetSyncState;

/// Discrete pet states; coordinates and animation frames stay local.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PetState {
    Away,
    Celebrate,
    Fight,
    Idle,
    Jump,
    Notification,
    Offline,
    Play,
    Run,
    Sleep,
    Typing,
    Walk,
}

impl PetState {
    /// Returns the synchronized representation when this state is network-visible.
    #[must_use]
    pub const fn sync_state(self) -> Option<PetSyncState> {
        match self {
            Self::Away => Some(PetSyncState::Away),
            Self::Celebrate => Some(PetSyncState::Celebrate),
            Self::Notification => Some(PetSyncState::Notification),
            Self::Offline => Some(PetSyncState::Offline),
            Self::Play => Some(PetSyncState::Play),
            Self::Sleep => Some(PetSyncState::Sleep),
            Self::Typing => Some(PetSyncState::Typing),
            Self::Fight | Self::Idle | Self::Jump | Self::Run | Self::Walk => None,
        }
    }
}

/// Local state and outbound synchronization policy for one pet instance.
#[derive(Debug)]
pub struct PetInstance {
    id: Uuid,
    last_broadcast: Option<Instant>,
    last_broadcast_state: Option<PetState>,
    state: PetState,
}

impl PetInstance {
    /// Creates a local pet in its idle state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            last_broadcast: None,
            last_broadcast_state: None,
            state: PetState::Idle,
        }
    }

    /// Returns the stable local instance ID.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the current local state.
    #[must_use]
    pub const fn state(&self) -> PetState {
        self.state
    }

    /// Changes local state and reports whether a discrete event may be broadcast.
    ///
    /// # Errors
    ///
    /// Returns an error when a new state exceeds the two-per-second limit.
    pub fn set_state(&mut self, state: PetState, now: Instant) -> Result<bool, PetStateError> {
        self.state = state;
        if self.last_broadcast_state == Some(state) {
            return Ok(false);
        }
        if self
            .last_broadcast
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(500))
        {
            return Err(PetStateError::RateLimited);
        }
        self.last_broadcast = Some(now);
        self.last_broadcast_state = Some(state);
        Ok(true)
    }
}

impl Default for PetInstance {
    fn default() -> Self {
        Self::new()
    }
}

/// Pet state transition failures.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum PetStateError {
    #[error("pet state broadcast exceeds two events per second")]
    RateLimited,
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{PetInstance, PetState, PetStateError};

    #[test]
    fn deduplicates_and_rate_limits_broadcasts() {
        let mut pet = PetInstance::new();
        let now = Instant::now();
        assert!(
            pet.set_state(PetState::Typing, now)
                .expect("first state broadcasts")
        );
        assert!(
            !pet.set_state(PetState::Typing, now)
                .expect("same state deduplicates")
        );
        assert_eq!(
            pet.set_state(PetState::Sleep, now),
            Err(PetStateError::RateLimited)
        );
        assert!(
            pet.set_state(PetState::Sleep, now + Duration::from_millis(500))
                .expect("rate window passes")
        );
        assert!(PetState::Typing.sync_state().is_some());
        assert!(PetState::Walk.sync_state().is_none());
    }
}
