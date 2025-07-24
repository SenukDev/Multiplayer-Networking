use uuid::Uuid;

#[derive(Debug)]
pub enum ServerToWorld {
    PlayerJoined { connection_id: Uuid },
    PlayerInput { connection_id: Uuid, input: String },
}

#[derive(Debug)]
pub enum WorldToServer {
    SendToClient { connection_id: Uuid, message: String },
    DisconnectClient { connection_id: Uuid },
}