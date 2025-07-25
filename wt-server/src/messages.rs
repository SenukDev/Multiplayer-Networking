use uuid::Uuid;

#[derive(Debug)]
pub enum ServerToWorld {
    PlayerJoined { connection_id: Uuid },
    InputClickPressed { connection_id: Uuid, x: f32, y: f32},
}

#[derive(Debug)]
pub enum WorldToServer {
    SendTick { receiver_connection_id: Uuid, tick: u64 },
    CreatePlayer { receiver_connection_id: Uuid, connection_id: Uuid, x: f32, y: f32},
    UpdatePlayerPosition { receiver_connection_id: Uuid, connection_id: Uuid, x: f32, y: f32}
}