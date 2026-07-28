#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command IPC deserializes owned arguments and State"
)]

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zenpaws_database::{DatabaseTrustStore, LamportCursor};
use zenpaws_network::{
    Envelope, LamportClock, MessageBody, MessagePayload, NetworkDiagnostics, NetworkService,
    TlsIdentity,
};
use zenpaws_settings::{LocalIdentity, load_local_identity, save_local_identity};
use zenpaws_shared::{EventBus, PeerId, PeerTrustStore};

struct NetworkRuntime {
    service: Arc<NetworkService>,
    shutdown: tokio::sync::watch::Sender<bool>,
}

static NETWORK_RUNTIME: OnceLock<NetworkRuntime> = OnceLock::new();

/// Details returned after the local LAN listener begins.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub peer_id: String,
    pub tcp_port: u16,
}

/// Starts the local encrypted LAN listener exactly once.
#[tauri::command]
pub async fn start_network(
    app: AppHandle,
    database: State<'_, crate::DatabaseState>,
    username: String,
) -> Result<NetworkStatus, String> {
    if let Some(runtime) = NETWORK_RUNTIME.get() {
        return status_for(&runtime.service);
    }

    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    let path = data_dir.join("network-identity.json");
    let (identity, tls_identity) = load_or_create_identity(&path, username)?;
    let last_seen_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .upsert_peer(
            identity.peer_id(),
            identity.username(),
            &tls_identity.pin().as_bytes(),
            last_seen_at,
        )
        .map_err(|error| error.to_string())?;
    let service = NetworkService::bind(
        SocketAddr::from(([0, 0, 0, 0], 0)),
        identity.peer_id(),
        identity.username().to_owned(),
        &tls_identity,
        EventBus::new(256),
        Duration::from_secs(30),
    )
    .await
    .map_err(|error| error.to_string())?;
    let trust_store: Arc<dyn PeerTrustStore> =
        Arc::new(DatabaseTrustStore::new(Arc::clone(&database.0)));
    let service = Arc::new(service);
    let status = status_for(&service)?;

    let (shutdown, receiver) = tokio::sync::watch::channel(false);
    NETWORK_RUNTIME
        .set(NetworkRuntime {
            service: Arc::clone(&service),
            shutdown,
        })
        .map_err(|_| "network service started concurrently".to_owned())?;
    tokio::spawn(Arc::clone(&service).run_inbound(receiver.clone()));
    tokio::spawn(Arc::clone(&service).run_discovery(Arc::clone(&trust_store), receiver.clone()));
    tokio::spawn(service.run_udp(trust_store, receiver));
    Ok(status)
}

/// Returns discovery, socket, and connected-peer diagnostics.
#[tauri::command]
pub fn network_diagnostics() -> Result<NetworkDiagnostics, String> {
    NETWORK_RUNTIME
        .get()
        .ok_or_else(|| "network service is not running".to_owned())?
        .service
        .diagnostics()
        .map_err(|error| error.to_string())
}

/// Stops LAN discovery and listener lifecycle tasks.
#[tauri::command]
pub fn stop_network() -> Result<(), String> {
    let runtime = NETWORK_RUNTIME
        .get()
        .ok_or_else(|| "network service is not running".to_owned())?;
    runtime
        .shutdown
        .send(true)
        .map_err(|error| error.to_string())?;
    runtime
        .service
        .shutdown()
        .map_err(|error| error.to_string())
}

pub(super) fn local_peer_id() -> Result<PeerId, String> {
    NETWORK_RUNTIME
        .get()
        .map(|runtime| runtime.service.local_peer_id())
        .ok_or_else(|| "network service is not running".to_owned())
}

pub(super) fn broadcast(envelope: Envelope) -> Result<(), String> {
    let runtime = NETWORK_RUNTIME
        .get()
        .ok_or_else(|| "network service is not running".to_owned())?;
    runtime.service.broadcast(&envelope);
    Ok(())
}

pub(crate) fn handle_sync_request(
    database: &Arc<Mutex<zenpaws_database::Database>>,
    peer_id: PeerId,
    room: String,
    since: LamportCursor,
) -> Result<(), String> {
    let messages = database
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .messages_after_lamport(&room, since, 200)
        .map_err(|error| error.to_string())?;
    let runtime = NETWORK_RUNTIME
        .get()
        .ok_or_else(|| "network service is not running".to_owned())?;
    for message in messages {
        runtime
            .service
            .send_to(
                peer_id,
                &Envelope::Message(MessagePayload {
                    author: message.author,
                    body: MessageBody::Text(message.body),
                    clock: LamportClock {
                        counter: message.lamport_counter,
                        peer_id: message.lamport_peer,
                    },
                    created_at: message.created_at,
                    id: message.id,
                    mentions: Vec::new(),
                    reply_to: message.reply_to,
                    room: message.room,
                }),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn status_for(service: &NetworkService) -> Result<NetworkStatus, String> {
    let address = service.local_addr().map_err(|error| error.to_string())?;
    Ok(NetworkStatus {
        peer_id: service.local_peer_id().as_uuid().to_string(),
        tcp_port: address.port(),
    })
}

fn load_or_create_identity(
    path: &std::path::Path,
    username: String,
) -> Result<(LocalIdentity, TlsIdentity), String> {
    if let Some(identity) = load_local_identity(path).map_err(|error| error.to_string())? {
        let tls = TlsIdentity::from_der(
            identity.certificate_der().to_vec(),
            identity.private_key_der().to_vec(),
        );
        return Ok((identity, tls));
    }

    let peer_id = PeerId::new();
    let tls = TlsIdentity::generate(format!("zenpaws-{}.local", peer_id.as_uuid()))
        .map_err(|error| error.to_string())?;
    let identity = LocalIdentity::new(
        peer_id,
        username,
        tls.certificate().as_ref().to_vec(),
        tls.private_key_der(),
    )
    .map_err(|error| error.to_string())?;
    save_local_identity(path, &identity).map_err(|error| error.to_string())?;
    Ok((identity, tls))
}
