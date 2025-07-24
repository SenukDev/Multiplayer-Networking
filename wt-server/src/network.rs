use uuid::Uuid;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MessageType {
    Tick = 0,
    CreatePlayer = 1,
}

impl MessageType {
    fn to_u8(self) -> u8 {
        self as u8
    }
}


pub fn build_tick_datagram(tick: u64) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 8);

    buffer.push(MessageType::Tick.to_u8());

    buffer.extend_from_slice(&tick.to_le_bytes());

    buffer
}

pub fn build_create_player_datagram(connection_id: Uuid, x: f32, y: f32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 16 + 4 + 4);

    buffer.push(MessageType::CreatePlayer.to_u8());

    buffer.extend_from_slice(connection_id.as_bytes());
    buffer.extend_from_slice(&x.to_le_bytes());
    buffer.extend_from_slice(&y.to_le_bytes());

    buffer
}