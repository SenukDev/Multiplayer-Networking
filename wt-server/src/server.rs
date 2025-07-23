use anyhow::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use std::time::Duration;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::Instrument;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;
use wtransport::endpoint::IncomingSession;
use wtransport::Endpoint;
use wtransport::Identity;
use wtransport::ServerConfig;
use uuid::Uuid;
use dashmap::DashMap;
use std::sync::Arc;
use crate::messages::{ServerToWorld, WorldToServer};

type ConnectionId = Uuid;

type ConnectionMap = Arc<DashMap<ConnectionId, wtransport::Connection>>;



pub async fn run_server(
    to_world: UnboundedSender<ServerToWorld>,
    mut from_world: UnboundedReceiver<WorldToServer>,
) -> Result<()> {
    init_logging();

    let connections: ConnectionMap = Arc::new(DashMap::new());

    // Server configuration
    let config = ServerConfig::builder()
        .with_bind_default(8443)
        .with_identity(Identity::load_pemfiles("cert.pem", "key.pem").await?)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let server = Endpoint::server(config)?;
    info!("Server ready!");

    loop {
        tokio::select! {
            // Accept new incoming session
            incoming_session = server.accept() => {
                let connection_id = Uuid::new_v4();
                tokio::spawn(handle_connection(incoming_session, connections.clone(), connection_id).instrument(info_span!("Connection", %connection_id)));
            }

            // // Process messages from the world
            // Some(msg) = from_world.recv() => {
            //     match msg {
            //         WorldToServer::SendToClient { session_id, message } => {
            //             println!("Send to client {}: {}", session_id, message);
            //             // TODO: lookup session and send the message
            //         }
            //         WorldToServer::DisconnectClient { session_id } => {
            //             println!("Disconnecting client {}", session_id);
            //             // TODO: remove session from session map
            //         }
            //     }
            // }

            // else => {
            //     // If both channels are closed or unreachable
            //     break;
            // }
        }
    }
}


async fn handle_connection(incoming_session: IncomingSession, connections: ConnectionMap, connection_id: ConnectionId) {
    let result = handle_connection_impl(incoming_session, connections, connection_id).await;
    error!("{:?}", result);
}

async fn handle_connection_impl(incoming_session: IncomingSession, connections: ConnectionMap, connection_id: ConnectionId) -> Result<()> {
    let mut buffer = vec![0; 65536].into_boxed_slice();

    info!("Waiting for session request...");

    let session_request = incoming_session.await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;

    connections.insert(connection_id, connection.clone());

    info!("---------------");
    info!("Connection Map");
    for entry in connections.iter() {
        let id = entry.key();
        info!("Connection ID: {}", id);
    }

    info!("Waiting for data from client...");

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let mut stream = stream?;
                info!("Accepted BI stream");
                
                let bytes_read = match stream.1.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (bi) '{str_data}' from client");

                stream.0.write_all(b"ACK").await?;
            }
            stream = connection.accept_uni() => {
                let mut stream = stream?;
                info!("Accepted UNI stream");

                let bytes_read = match stream.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (uni) '{str_data}' from client");

                let mut stream = connection.open_uni().await?.await?;
                stream.write_all(b"ACK").await?;
            }
            dgram = connection.receive_datagram() => {
                // let dgram = dgram?;
                // let str_data = std::str::from_utf8(&dgram)?;

                // info!("Received (dgram) '{str_data}' from client");

                // connection.send_datagram(b"ACK")?;
            }
        }
    }
}

fn init_logging() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter(env_filter)
        .init();
}