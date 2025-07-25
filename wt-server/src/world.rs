use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::messages::{ServerToWorld, WorldToServer};
use tokio::time::{interval, Duration};
use anyhow::Result;


use hecs::World;
use crate::components::*;
use crate::systems::*;

pub async fn run_world(
    mut from_server: UnboundedReceiver<ServerToWorld>,
    to_server: UnboundedSender<WorldToServer>,
) -> Result<()> {
    let mut tick = interval(Duration::from_secs_f64(1.0 / 30.0));

    //Initialise World
    let mut world = World::new();
    world.spawn((Tick { tick: 0 },));

    world.spawn((
        Collision {
            collision_lines: vec![
                CollisionLine { x1: 192.0, y1: 128.0, x2: 320.0, y2: 128.0 },
                CollisionLine { x1: 320.0, y1: 128.0, x2: 320.0, y2: 256.0 },
                CollisionLine { x1: 320.0, y1: 256.0, x2: 296.0, y2: 208.0 },
                CollisionLine { x1: 296.0, y1: 208.0, x2: 248.0, y2: 256.0 },
            ]
        },
    ));
    
    loop {
        tick.tick().await;

        update_tick(&mut world, to_server.clone());

        // Process messages from the world
        while let Ok(msg) = from_server.try_recv() {
            match msg {
                ServerToWorld::PlayerJoined { connection_id } => {

                    create_player(&mut world, to_server.clone(), connection_id, 256.0, 192.0);
                }
                ServerToWorld::InputClickPressed { connection_id, x, y } => {
                    input_click_pressed(&mut world, connection_id, x, y);
                }
            }
        }

        update_state(&mut world);
        handle_state(&mut world);
        apply_velocity(&mut world);
        broadcast_positions(&mut world, to_server.clone());
    }
}