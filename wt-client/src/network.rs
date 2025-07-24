use uuid::Uuid;
use hecs::World;
use log::info;
use crate::systems::*;



#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Tick = 0,
    CreatePlayer = 1,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MessageType::Tick),
            1 => Some(MessageType::CreatePlayer),
            _ => None,
        }
    }
}

pub fn decode_tick_datagram(data: &[u8], world: &mut World) {
    if data.len() < 9 {
        web_sys::console::error_1(&"Tick datagram too short".into());
        return;
    }

    let tick_bytes: [u8; 8] = data[1..9].try_into().unwrap();
    let tick = u64::from_be_bytes(tick_bytes);

    // Update tick in ECS TODO
}

pub fn decode_create_player_datagram(data: &[u8], world: &mut World) {
    if data.len() < 1 + 16 + 4 + 4 {
        web_sys::console::error_1(&"CreatePlayer datagram too short".into());
        return;
    }

    let uuid_bytes = &data[1..17];
    let uuid = match Uuid::from_slice(uuid_bytes) {
        Ok(id) => id,
        Err(_) => {
            web_sys::console::error_1(&"Invalid UUID".into());
            return;
        }
    };

    let x = f32::from_le_bytes(data[17..21].try_into().unwrap());
    let y = f32::from_le_bytes(data[21..25].try_into().unwrap());
    
    info!("Player {} Created : ({}, {})", uuid, x, y);

    create_player(world, uuid, x, y);
}