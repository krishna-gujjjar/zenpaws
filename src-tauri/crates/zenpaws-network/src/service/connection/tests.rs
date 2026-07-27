use std::sync::Arc;

use rustls::pki_types::ServerName;
use tokio::net::TcpListener;
use uuid::Uuid;
use zenpaws_shared::{EventBus, PeerId};

use super::PeerConnection;
use crate::{
    ConnectionStateMachine, Envelope, HandshakeIdentity, MessageBody, MessagePayload, TlsIdentity,
    accept_tls, client_handshake, connect_tls, read_envelope, server_handshake,
};

#[tokio::test]
async fn broadcasts_a_message_over_one_loopback_peer() {
    let server_id = PeerId::new();
    let client_id = PeerId::new();
    let server_identity = TlsIdentity::generate("localhost".to_owned()).expect("server TLS");
    let client_identity =
        HandshakeIdentity::new(client_id, "client".to_owned()).expect("client identity");
    let server_handshake_identity =
        HandshakeIdentity::new(server_id, "server".to_owned()).expect("server identity");
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener binds");
    let address = listener.local_addr().expect("listener address");
    let server_config = server_identity.server_config().expect("server config");
    let bus = EventBus::new(8);

    let server = tokio::spawn(async move {
        let (mut stream, address) = accept_tls(&listener, server_config).await.expect("accepts");
        let remote = server_handshake(&mut stream, &server_handshake_identity)
            .await
            .expect("server handshake");
        let mut state = ConnectionStateMachine::new(remote.peer_id(), bus);
        state
            .connected(std::time::Duration::from_secs(30))
            .expect("connects");
        PeerConnection {
            address,
            heartbeat_timeout: std::time::Duration::from_secs(30),
            remote,
            state,
            stream,
        }
        .start()
    });

    let client_config =
        TlsIdentity::client_config_for_peer(server_identity.certificate()).expect("client config");
    let client_stream = connect_tls(
        address,
        ServerName::try_from("localhost")
            .expect("server name")
            .to_owned(),
        Arc::clone(&client_config),
    )
    .await
    .expect("client connects");
    let mut client_stream = client_stream;
    client_handshake(&mut client_stream, &client_identity, server_id)
        .await
        .expect("client handshake");
    let sender = server.await.expect("server task");
    let message = Envelope::Message(MessagePayload {
        author: server_id,
        body: MessageBody::Text("loopback message".to_owned()),
        clock: crate::LamportClock {
            counter: 1,
            peer_id: server_id,
        },
        created_at: 1,
        id: Uuid::new_v4(),
        mentions: Vec::new(),
        reply_to: None,
        room: "shared".to_owned(),
    });
    sender.send(message.clone()).await.expect("message queues");

    assert_eq!(
        read_envelope(&mut client_stream)
            .await
            .expect("message reads"),
        message
    );
}
