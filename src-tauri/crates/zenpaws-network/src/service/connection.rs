use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use tokio::{net::TcpStream, sync::mpsc};
use tokio_rustls::TlsStream;
use zenpaws_shared::{PeerId, ZenPawsEvent};

use crate::{
    ConnectionStateMachine, Envelope, HandshakeIdentity, MessageBody, ServiceError, read_envelope,
    write_envelope,
};

/// One accepted, TLS-protected peer connection.
pub struct PeerConnection {
    pub(super) address: SocketAddr,
    pub(super) heartbeat_timeout: Duration,
    pub(super) remote: HandshakeIdentity,
    pub(super) state: ConnectionStateMachine,
    pub(super) stream: TlsStream<TcpStream>,
}

impl PeerConnection {
    /// Returns the remote TCP endpoint.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns the identity validated by the application handshake.
    #[must_use]
    pub const fn peer_id(&self) -> PeerId {
        self.remote.peer_id()
    }

    /// Sends a heartbeat without allocating a polling task.
    ///
    /// # Errors
    ///
    /// Returns an error when encrypted framing fails.
    pub async fn send_heartbeat(&mut self) -> Result<(), ServiceError> {
        write_envelope(&mut self.stream, &Envelope::Heartbeat).await?;
        Ok(())
    }

    /// Receives one control envelope and refreshes heartbeat state when needed.
    ///
    /// # Errors
    ///
    /// Returns an error when encrypted framing fails or the peer state is invalid.
    pub async fn receive(&mut self) -> Result<Envelope, ServiceError> {
        let envelope = read_envelope(&mut self.stream).await?;
        if matches!(envelope, Envelope::Heartbeat) {
            self.state.heartbeat(self.heartbeat_timeout)?;
        }
        Ok(envelope)
    }

    /// Monitors inbound control frames until the peer disconnects or a frame fails.
    pub(super) fn start(self) -> mpsc::Sender<Envelope> {
        let (sender, receiver) = mpsc::channel(64);
        tokio::spawn(self.run(receiver));
        sender
    }

    async fn run(mut self, mut outbound: mpsc::Receiver<Envelope>) {
        loop {
            tokio::select! {
                result = self.receive() => {
                    match result {
                        Ok(Envelope::Message(payload)) => {
                            match payload.body {
                                MessageBody::Text(body) => self.state.publish(ZenPawsEvent::MessageReceived {
                                    message_id: payload.id,
                                    room: payload.room,
                                    author: payload.author,
                                    author_name: self.remote.username().to_owned(),
                                    body,
                                    created_at: payload.created_at,
                                    lamport_counter: payload.clock.counter,
                                    reply_to: payload.reply_to,
                                }),
                                MessageBody::Edit { message_id, body } => self.state.publish(ZenPawsEvent::MessageEdited {
                                    message_id,
                                    body,
                                    edited_at: payload.created_at,
                                    lamport_counter: payload.clock.counter,
                                    lamport_peer: payload.clock.peer_id,
                                }),
                                MessageBody::Delete { message_id } => self.state.publish(ZenPawsEvent::MessageDeleted {
                                    message_id,
                                    deleted_at: payload.created_at,
                                    lamport_counter: payload.clock.counter,
                                    lamport_peer: payload.clock.peer_id,
                                }),
                                MessageBody::Reaction { message_id, emoji } => self.state.publish(ZenPawsEvent::MessageReactionAdded {
                                    message_id,
                                    peer_id: payload.author,
                                    emoji,
                                }),
                            }
                            if write_envelope(
                                &mut self.stream,
                                &Envelope::Ack {
                                    message_id: payload.id,
                                    kind: crate::AckKind::Delivered,
                                },
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                        }
                        Ok(Envelope::Ack { message_id, kind }) => {
                            self.state.publish(ZenPawsEvent::MessageAckReceived {
                                message_id,
                                peer_id: self.remote.peer_id(),
                                read: matches!(kind, crate::AckKind::Read),
                            });
                        }
                        Ok(Envelope::SyncRequest { room, since }) => {
                            self.state.publish(ZenPawsEvent::SyncRequested {
                                peer_id: self.remote.peer_id(),
                                room,
                                since_counter: since.counter,
                                since_peer: since.peer_id,
                            });
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
                envelope = outbound.recv() => {
                    match envelope {
                        Some(envelope) if write_envelope(&mut self.stream, &envelope).await.is_ok() => {}
                        _ => break,
                    }
                }
            }
        }
        let _ = self.state.disconnected();
    }

    /// Applies the configured heartbeat deadline.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection state cannot transition.
    pub fn check_heartbeat(&mut self, now: Instant) -> Result<(), ServiceError> {
        self.state.check_heartbeat(now)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
