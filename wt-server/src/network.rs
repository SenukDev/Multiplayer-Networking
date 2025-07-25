use uuid::Uuid;
use crate::messages::ServerToWorld;
use tokio::sync::mpsc::UnboundedSender;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ServerToClientMessage {
    Tick = 0,
    CreatePlayer = 1,
    UpdatePlayerPosition = 2,
}

impl ServerToClientMessage {
    fn to_u8(self) -> u8 {
        self as u8
    }
}


pub fn build_tick_datagram(tick: u64) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 8);

    buffer.push(ServerToClientMessage::Tick.to_u8());

    buffer.extend_from_slice(&tick.to_le_bytes());

    buffer
}

pub fn build_create_player_datagram(connection_id: Uuid, x: f32, y: f32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 16 + 4 + 4);

    buffer.push(ServerToClientMessage::CreatePlayer.to_u8());

    buffer.extend_from_slice(connection_id.as_bytes());
    buffer.extend_from_slice(&x.to_le_bytes());
    buffer.extend_from_slice(&y.to_le_bytes());

    buffer
}

pub fn build_update_player_position_datagram(connection_id: Uuid, x: f32, y: f32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 16 + 4 + 4);

    buffer.push(ServerToClientMessage::UpdatePlayerPosition.to_u8());

    buffer.extend_from_slice(connection_id.as_bytes());
    buffer.extend_from_slice(&x.to_le_bytes());
    buffer.extend_from_slice(&y.to_le_bytes());

    buffer
}


#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClientToServerMessage {
    InputClickPressed = 0,
}

impl ClientToServerMessage {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ClientToServerMessage::InputClickPressed),
            _ => None,
        }
    }
}

pub fn decode_input_click_pressed(connection_id: Uuid, to_world: UnboundedSender<ServerToWorld>, data: &[u8]) {
    if data.len() < 1 + 4 + 4 {
        return;
    }

    let x = f32::from_le_bytes(data[1..5].try_into().unwrap());
    let y = f32::from_le_bytes(data[5..9].try_into().unwrap());
    
    println!("Player {} Clicked at: {} {})", connection_id, x, y);
    to_world.send(ServerToWorld::InputClickPressed { connection_id: connection_id, x: x, y: y }).unwrap();
}