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
use crate::network::*;
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
                tokio::spawn(handle_connection(
                    incoming_session,
                    connections.clone(),
                    connection_id,
                    to_world.clone(),
                ).instrument(info_span!("Connection", %connection_id)));
            }
            // Process messages from the world
            Some(msg) = from_world.recv() => {
                match msg {
                    WorldToServer::SendTick { receiver_connection_id, tick } => {
                        if let Some(connection) = connections.get(&receiver_connection_id) {
                            let message = build_tick_datagram(tick);
                            connection.send_datagram(message)?;
                        }
                    }
                    WorldToServer::CreatePlayer { receiver_connection_id, connection_id, x, y } => {
                        if let Some(connection) = connections.get(&receiver_connection_id) {
                            let message = build_create_player_datagram(connection_id, x, y);
                            let mut stream = connection.open_uni().await?.await?;
                            stream.write_all(&message).await?;
                        }
                    }
                    WorldToServer::UpdatePlayerPosition { receiver_connection_id, connection_id, x, y } => {
                        if let Some(connection) = connections.get(&receiver_connection_id) {
                            let message = build_update_player_position_datagram(connection_id, x, y);
                            connection.send_datagram(message)?;
                        }
                    }
                }
            }
            else => {
                // If both channels are closed or unreachable
                return Ok(());
            }
        }
    }
}


async fn handle_connection(
    incoming_session: IncomingSession,
    connections: ConnectionMap,
    connection_id: ConnectionId,
    to_world: UnboundedSender<ServerToWorld>,
) {
    let result = handle_connection_impl(incoming_session, connections, connection_id, to_world).await;
    error!("{:?}", result);
}

async fn handle_connection_impl(
    incoming_session: IncomingSession,
    connections: ConnectionMap,
    connection_id: ConnectionId,
    to_world: UnboundedSender<ServerToWorld>,
) -> Result<()> {
    let mut buffer = vec![0; 65536].into_boxed_slice();

    let session_request = incoming_session.await?;

    let connection = session_request.accept().await?;

    connections.insert(connection_id, connection.clone());
    to_world.send(ServerToWorld::PlayerJoined { connection_id })?;

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
                let dgram = dgram?;

                if dgram.is_empty() {
                    info!("Empty datagram received");
                } else {
                    match ClientToServerMessage::from_u8(dgram[0]) {
                        Some(ClientToServerMessage::InputClickPressed) => {
                            decode_input_click_pressed(connection_id, to_world.clone(), &dgram);
                        }

                        None => {
                            info!("Unknown message type: {}", dgram[0]);
                        }
                    }
                }
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