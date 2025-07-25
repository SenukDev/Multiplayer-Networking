use uuid::Uuid;
use hecs::World;
use log::info;
use crate::systems::*;



#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServerToClientMessage {
    Tick = 0,
    CreatePlayer = 1,
    UpdatePlayerPosition = 2,
}

impl ServerToClientMessage {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ServerToClientMessage::Tick),
            1 => Some(ServerToClientMessage::CreatePlayer),
            2 => Some(ServerToClientMessage::UpdatePlayerPosition),
            _ => None,
        }
    }
}

pub fn decode_tick_datagram(data: &[u8], world: &mut World) {
    if data.len() < 9 {
        return;
    }

    //let tick_bytes: [u8; 8] = data[1..9].try_into().unwrap();
    //let tick = u64::from_be_bytes(tick_bytes);

    // Update tick in ECS TODO
}

pub fn decode_create_player_datagram(data: &[u8], world: &mut World) {
    if data.len() < 1 + 16 + 4 + 4 {
        return;
    }

    let uuid_bytes = &data[1..17];
    let uuid = match Uuid::from_slice(uuid_bytes) {
        Ok(id) => id,
        Err(_) => {
            return;
        }
    };

    let x = f32::from_le_bytes(data[17..21].try_into().unwrap());
    let y = f32::from_le_bytes(data[21..25].try_into().unwrap());
    
    info!("Player {} Created : ({}, {})", uuid, x, y);

    create_player(world, uuid, x, y);
}

pub fn decode_update_player_position_datagram(data: &[u8], world: &mut World) {
    if data.len() < 1 + 16 + 4 + 4 {
        return;
    }

    let uuid_bytes = &data[1..17];
    let uuid = match Uuid::from_slice(uuid_bytes) {
        Ok(id) => id,
        Err(_) => {
            return;
        }
    };

    let x = f32::from_le_bytes(data[17..21].try_into().unwrap());
    let y = f32::from_le_bytes(data[21..25].try_into().unwrap());

    update_position(world, uuid, x, y);
}


#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ClientToServerMessage {
    InputClickPressed = 0,
}

impl ClientToServerMessage {
    fn to_u8(self) -> u8 {
        self as u8
    }
}

pub fn build_input_click_pressed(x: f32, y: f32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 16 + 4 + 4);

    buffer.push(ClientToServerMessage::InputClickPressed.to_u8());
    
    buffer.extend_from_slice(&x.to_le_bytes());
    buffer.extend_from_slice(&y.to_le_bytes());

    buffer
}