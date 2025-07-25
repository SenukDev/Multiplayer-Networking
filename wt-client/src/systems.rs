use uuid::Uuid;
use hecs::World;
use crate::components::*;
use log::info;

pub fn update_tick(world: &mut World) {
    for (_, tick) in world.query_mut::<&mut Tick>() {
        tick.tick += 1;
    }
}

pub fn create_player(world: &mut World, connection_id: Uuid, x: f32, y: f32) {
    world.spawn((
        Player,
        Connection {connection_id: connection_id},
        Position { x, y },
        PlayerCollision { radius: 16.0, offset_x: 0.0, offset_y: 0.0 },
    ));
}

pub fn update_position(world: &mut World, connection_id: Uuid, x: f32, y: f32) {
    for (_,(
        _,
        connection,
        position,
    )) in world.query_mut::<(
        &Player,
        &Connection,
        &mut Position,
    )>() {
        if connection.connection_id == connection_id {
            position.x = x;
            position.y = y;
        }
    }
}