use std::{net::SocketAddr, sync::Arc};

use rustls::{ClientConfig, ServerConfig, pki_types::ServerName};
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};

/// Accepts one TCP connection and upgrades it to TLS.
///
/// # Errors
///
/// Returns an error when accepting the TCP socket or completing the TLS
/// handshake fails.
pub async fn accept_tls(
    listener: &TcpListener,
    server_config: Arc<ServerConfig>,
) -> Result<(TlsStream<TcpStream>, SocketAddr), TransportError> {
    let (stream, address) = listener.accept().await?;
    let stream = TlsAcceptor::from(server_config).accept(stream).await?;
    Ok((TlsStream::Server(stream), address))
}

/// Connects to a peer over TCP and completes a pinned TLS handshake.
///
/// # Errors
///
/// Returns an error when TCP connection, server-name validation, or the TLS
/// handshake fails.
pub async fn connect_tls(
    address: SocketAddr,
    server_name: ServerName<'static>,
    client_config: Arc<ClientConfig>,
) -> Result<TlsStream<TcpStream>, TransportError> {
    let stream = TcpStream::connect(address).await?;
    let stream = TlsConnector::from(client_config)
        .connect(server_name, stream)
        .await?;
    Ok(TlsStream::Client(stream))
}

/// Encrypted transport failures.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("TCP transport I/O failed")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rustls::pki_types::ServerName;
    use tokio::net::TcpListener;

    use super::{accept_tls, connect_tls};
    use crate::TlsIdentity;

    #[tokio::test]
    async fn accepts_a_pinned_tls_connection() {
        let server_identity =
            TlsIdentity::generate("localhost".to_owned()).expect("server identity is generated");
        let server_config = server_identity
            .server_config()
            .expect("server config is valid");
        let client_config = TlsIdentity::client_config_for_peer(server_identity.certificate())
            .expect("client config is valid");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener binds");
        let address = listener.local_addr().expect("listener has an address");
        let server = tokio::spawn(async move { accept_tls(&listener, server_config).await });
        let name = ServerName::try_from("localhost")
            .expect("server name is valid")
            .to_owned();

        let client = connect_tls(address, name, Arc::clone(&client_config)).await;
        let accepted = server.await.expect("server task completes");

        assert!(client.is_ok());
        assert!(accepted.is_ok());
    }
}
