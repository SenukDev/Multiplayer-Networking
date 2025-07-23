use uuid::Uuid;

#[derive(Debug)]
pub enum ServerToWorld {
    PlayerJoined { session_id: u64 },
    PlayerInput { session_id: u64, input: String },
}

#[derive(Debug)]
pub enum WorldToServer {
    SendToClient { session_id: u64, message: String },
    DisconnectClient { session_id: u64 },
}