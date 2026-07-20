use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zenpaws_database::DatabaseTrustStore;
use zenpaws_network::{NetworkService, TlsIdentity};
use zenpaws_settings::{LocalIdentity, load_local_identity, save_local_identity};
use zenpaws_shared::{EventBus, PeerId};

struct NetworkRuntime {
    service: Arc<NetworkService>,
    shutdown: tokio::sync::watch::Sender<bool>,
}

static NETWORK_RUNTIME: OnceLock<NetworkRuntime> = OnceLock::new();

/// Details returned after the local LAN listener begins.
#[derive(Debug, Serialize)]
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
    let trust_store = Arc::new(DatabaseTrustStore::new(Arc::clone(&database.0)));
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
