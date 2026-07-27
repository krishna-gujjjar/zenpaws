use std::time::{Duration, Instant};

use thiserror::Error;
use zenpaws_shared::{ConnectionState, EventBus, PeerId, ZenPawsEvent};

/// Tracks one peer's lifecycle without performing transport I/O.
#[derive(Debug)]
pub struct ConnectionStateMachine {
    bus: EventBus,
    heartbeat_deadline: Option<Instant>,
    peer_id: PeerId,
    state: ConnectionState,
}

impl ConnectionStateMachine {
    /// Creates a peer session in the connecting state.
    #[must_use]
    pub const fn new(peer_id: PeerId, bus: EventBus) -> Self {
        Self {
            bus,
            heartbeat_deadline: None,
            peer_id,
            state: ConnectionState::Connecting,
        }
    }

    /// Publishes a cross-module event from this peer session.
    pub fn publish(&self, event: ZenPawsEvent) {
        self.bus.publish(event);
    }

    /// Returns the current peer state.
    #[must_use]
    pub const fn state(&self) -> ConnectionState {
        self.state
    }

    /// Records a successful TLS and TOFU handshake.
    ///
    /// # Errors
    ///
    /// Returns an error when the current state cannot become connected.
    pub fn connected(&mut self, heartbeat_timeout: Duration) -> Result<(), StateError> {
        self.transition(ConnectionState::Connected)?;
        self.heartbeat_deadline = Some(Instant::now() + heartbeat_timeout);
        Ok(())
    }

    /// Records a heartbeat and refreshes the degradation deadline.
    ///
    /// # Errors
    ///
    /// Returns an error when the peer is disconnected or a transition fails.
    pub fn heartbeat(&mut self, heartbeat_timeout: Duration) -> Result<(), StateError> {
        if self.state == ConnectionState::Disconnected {
            return Err(StateError::HeartbeatWhileDisconnected);
        }
        self.heartbeat_deadline = Some(Instant::now() + heartbeat_timeout);
        if self.state == ConnectionState::Degraded {
            self.transition(ConnectionState::Connected)?;
        }
        Ok(())
    }

    /// Applies the heartbeat deadline to the state machine.
    ///
    /// # Errors
    ///
    /// Returns an error when the current state cannot become degraded.
    pub fn check_heartbeat(&mut self, now: Instant) -> Result<(), StateError> {
        if self
            .heartbeat_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            self.transition(ConnectionState::Degraded)?;
        }
        Ok(())
    }

    /// Marks the peer unavailable after transport closure or retry exhaustion.
    ///
    /// # Errors
    ///
    /// Returns an error when the current state cannot become disconnected.
    pub fn disconnected(&mut self) -> Result<(), StateError> {
        self.heartbeat_deadline = None;
        self.transition(ConnectionState::Disconnected)
    }

    fn transition(&mut self, next: ConnectionState) -> Result<(), StateError> {
        if !is_valid_transition(self.state, next) {
            return Err(StateError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        if self.state != next {
            self.state = next;
            self.bus.publish(ZenPawsEvent::PeerStateChanged {
                peer_id: self.peer_id,
                state: next,
            });
        }
        Ok(())
    }
}

fn is_valid_transition(from: ConnectionState, to: ConnectionState) -> bool {
    matches!(
        (from, to),
        (
            ConnectionState::Connecting | ConnectionState::Degraded,
            ConnectionState::Connected | ConnectionState::Disconnected
        ) | (
            ConnectionState::Connected,
            ConnectionState::Degraded | ConnectionState::Disconnected
        ) | (ConnectionState::Disconnected, ConnectionState::Connecting)
    ) || from == to
}

/// Invalid state machine operations.
#[derive(Debug, Error)]
pub enum StateError {
    #[error("cannot receive a heartbeat while disconnected")]
    HeartbeatWhileDisconnected,
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: ConnectionState,
        to: ConnectionState,
    },
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use zenpaws_shared::{ConnectionState, EventBus, PeerId};

    use super::ConnectionStateMachine;

    #[test]
    fn heartbeat_timeout_degrades_a_connected_peer() {
        let mut session = ConnectionStateMachine::new(PeerId::new(), EventBus::new(1));
        session
            .connected(Duration::from_millis(1))
            .expect("valid transition");
        session
            .check_heartbeat(Instant::now() + Duration::from_secs(1))
            .expect("timeout check is valid");

        assert_eq!(session.state(), ConnectionState::Degraded);
    }
}
