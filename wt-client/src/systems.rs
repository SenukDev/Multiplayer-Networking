use uuid::Uuid;
use hecs::World;
use crate::components::*;

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