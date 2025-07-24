use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use crate::messages::{ServerToWorld, WorldToServer};
use tokio::time::{interval, Duration};
use anyhow::Result;


use hecs::World;
use crate::components::*;
use crate::systems::*;
use rand::Rng;

pub async fn run_world(
    mut from_server: UnboundedReceiver<ServerToWorld>,
    to_server: UnboundedSender<WorldToServer>,
) -> Result<()> {
    let mut tick = interval(Duration::from_secs(1));

    //Initialise World
    let mut world = World::new();
    world.spawn((Tick { tick: 0 },));
    
    loop {
        tick.tick().await;

        update_tick(&mut world, to_server.clone());

        // Process messages from the world
        while let Ok(msg) = from_server.try_recv() {
            match msg {
                ServerToWorld::PlayerJoined { connection_id } => {
                    let mut rng = rand::thread_rng();
                    let random_offset_x: f32 = rng.gen_range(-128.0..=128.0);
                    let random_offset_y: f32 = rng.gen_range(-96.0..=96.0);

                    create_player(&mut world, to_server.clone(), connection_id, 256.0 + random_offset_x, 192.0 + random_offset_y);
                }
                ServerToWorld::PlayerInput { connection_id, input } => {
                    println!("Player {} Input: {}", connection_id, input);
                }
            }
        }
    }
}