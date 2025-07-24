use crate::components::*;
use hecs::World;
use uuid::Uuid;

use tokio::sync::mpsc::UnboundedSender;
use crate::messages::WorldToServer;

pub fn update_tick(world: &mut World, to_server: UnboundedSender<WorldToServer>) {
    let mut tick_value = 0;

    for (_, tick) in world.query_mut::<&mut Tick>() {
        tick.tick += 1;
        tick_value = tick.tick;
        println!("Server Tick: {} ", tick_value);
    }

    for (_, connection) in world.query_mut::<&Connection>() {
        to_server.send(WorldToServer::SendTick {
            receiver_connection_id: connection.connection_id,
            tick: tick_value,
        }).unwrap();
        
        //println!("Broadcasting Tick to {}", connection.connection_id);
    }
}

pub fn create_player(world: &mut World, to_server: UnboundedSender<WorldToServer>, connection_id: Uuid, x: f32, y: f32) {
    world.spawn((
        Player,
        Connection {connection_id: connection_id},
        Position { x: x, y: y},
    ));
    println!("Player {} Created at X {}, Y {}", connection_id, x, y);
    
    for (_,(
        connection,
        _,
        position,
    )) in world.query::<(
        &Connection,
        &Player,
        &Position,
    )>().iter() {
        //Create the new player for exisitng connections
        to_server.send(WorldToServer::CreatePlayer {
            receiver_connection_id: connection.connection_id,
            connection_id: connection_id,
            x: x,
            y: y,
        }).unwrap();
        
        //Create existing players to new player
        if connection_id != connection.connection_id {
            to_server.send(WorldToServer::CreatePlayer {
                receiver_connection_id: connection_id,
                connection_id: connection.connection_id,
                x: position.x,
                y: position.y,
            }).unwrap();
        }
    }
}