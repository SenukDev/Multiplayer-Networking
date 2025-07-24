use uuid::Uuid;

#[derive(Debug)]
pub enum ServerToWorld {
    PlayerJoined { connection_id: Uuid },
    PlayerInput { connection_id: Uuid, input: String },
}

#[derive(Debug)]
pub enum WorldToServer {
    SendTick { receiver_connection_id: Uuid, tick: u64 },
    CreatePlayer { receiver_connection_id: Uuid, connection_id: Uuid, x: f32, y: f32}
}