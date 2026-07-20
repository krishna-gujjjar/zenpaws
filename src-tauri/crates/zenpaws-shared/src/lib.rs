//! Shared types and the cross-crate event bus.
//!
//! See `docs/25_EVENT_BUS_ARCHITECTURE.md`.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

mod trust;

pub use trust::{CertificatePin, PeerTrustStore, TrustStoreError, TrustedPeer};

/// Stable peer identity independent of a peer's current IP address.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PeerId(Uuid);

impl PeerId {
    /// Creates a fresh peer identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }

    /// Creates an identity from a persisted or received UUID.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self::new()
    }
}

/// A state transition visible to other modules through the event bus.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Degraded,
    Disconnected,
}

/// Discrete network-visible pet states. Animation data remains local.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PetSyncState {
    Away,
    Celebrate,
    Notification,
    Offline,
    Play,
    Sleep,
    Typing,
}

/// Cross-module events. Feature-specific payloads remain in their owner crate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ZenPawsEvent {
    PeerStateChanged {
        peer_id: PeerId,
        state: ConnectionState,
    },
    PeerTrustViolation {
        peer_id: PeerId,
    },
    PetStateChanged {
        pet_instance: Uuid,
        state: PetSyncState,
    },
}

/// In-process fanout for decoupled backend modules.
#[derive(Clone, Debug)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<ZenPawsEvent>>,
}

impl EventBus {
    /// Creates a bus with a bounded event history for slow subscribers.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publishes an event to all active subscribers.
    pub fn publish(&self, event: ZenPawsEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribes to future events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<ZenPawsEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::{ConnectionState, EventBus, PeerId, ZenPawsEvent};

    #[tokio::test]
    async fn publishes_to_subscribers() {
        let bus = EventBus::new(1);
        let mut receiver = bus.subscribe();
        let event = ZenPawsEvent::PeerStateChanged {
            peer_id: PeerId::new(),
            state: ConnectionState::Connected,
        };

        bus.publish(event.clone());

        assert_eq!(receiver.recv().await.expect("event is available"), event);
    }
}
