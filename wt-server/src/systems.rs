use crate::components::*;
use crate::scripts::*;
use hecs::World;
use uuid::Uuid;

use tokio::sync::mpsc::UnboundedSender;
use crate::messages::WorldToServer;

pub fn update_tick(world: &mut World, to_server: UnboundedSender<WorldToServer>) {
    let mut tick_value = 0;

    for (_, tick) in world.query_mut::<&mut Tick>() {
        tick.tick += 1;
        tick_value = tick.tick;
        //println!("Server Tick: {} ", tick_value);
    }

    for (_, connection) in world.query_mut::<&Connection>() {
        to_server.send(WorldToServer::SendTick {
            receiver_connection_id: connection.connection_id,
            tick: tick_value,
        }).unwrap();
    }
}

pub fn create_player(world: &mut World, to_server: UnboundedSender<WorldToServer>, connection_id: Uuid, x: f32, y: f32) {
    world.spawn((
        Player,
        Connection {connection_id: connection_id},
        State {state: PlayerState::Idle},
        Position { x: x, y: y},
        Velocity { x: 0.0, y: 0.0 },
        MoveTarget {x: x, y: y},
        PlayerCollision { radius: 16.0, offset_x: 0.0, offset_y: 0.0 },
        PlayerMove {move_speed: 2.0, move_input_type: MovementType::Target, timer: 0, }, //timer_threshold: 10, direction_radius: 24.0
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

pub fn input_click_pressed(world: &mut World, connection_id: Uuid, x: f32, y: f32) {
    for (_,(
        connection,
        _,
        target,
        move_type,
    )) in world.query_mut::<(
        &Connection,
        &Player,
        &mut MoveTarget,
        &mut PlayerMove,
    )>() {
        if connection.connection_id == connection_id {
            move_type.move_input_type = MovementType::Target;
            move_type.timer = 0;
            target.x = x;
            target.y = y;
            break;
        }
    }
}

pub fn update_state(world: &mut World) {
    for (_,(
        _,
        state,
        position,
        target
    )) in world.query::<(
        &Player,
        &mut State,
        &Position,
        &MoveTarget,
    )>().iter() {
        let dx = target.x - position.x;
        let dy = target.y - position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            state.state = PlayerState::Move;
        }
        else {
            state.state = PlayerState::Idle;
        }
    }
}

pub fn handle_state(world: &mut World) {
    for (_,(
        _,
        state,
        position,
        velocity,
        target,
        player_collision,
        player_move,
    )) in world.query::<(
        &Player,
        &mut State,
        &Position,
        &mut Velocity,
        &mut MoveTarget,
        &PlayerCollision,
        &PlayerMove
    )>().iter() {
        match state.state {
            PlayerState::Idle => {
                target.x = position.x;
                target.y = position.y;
                velocity.x = 0.0;
                velocity.y = 0.0;
            },
            PlayerState::Move => {
                //Velocity towards Move Target
                let dx = target.x - position.x;
                let dy = target.y - position.y;
                let length = (dx * dx + dy * dy).sqrt();

                if length > player_move.move_speed {
                    velocity.x = dx / length * player_move.move_speed;
                    velocity.y = dy / length * player_move.move_speed;
                } else {
                    velocity.x = dx;
                    velocity.y = dy;
                }
                
                for (_, collision) in world.query::<&Collision>().iter() {
                    let (vx, vy) = collision_slide_velocity(&position, &velocity, &player_collision, &collision, 4);
                    velocity.x = vx;
                    velocity.y = vy;
                }

                if velocity.x == 0.0 && velocity.y == 0.0 {
                    target.x = position.x;
                    target.y = position.y;
                }
            }
        }
    }
}

pub fn apply_velocity(world: &mut World) {
    for (_,(
        _,
        position,
        velocity,
        _
    )) in world.query::<(
        &Player,
        &mut Position,
        &Velocity,
        &PlayerCollision
    )>().iter() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

pub fn broadcast_positions(world: &mut World, to_server: UnboundedSender<WorldToServer>) {
    for (_,(
        broadcast_connection,
    )) in world.query::<(
        &Connection,
    )>().iter() {
        for (_,(
            connection,
            _,
            position,
        )) in world.query::<(
            &Connection,
            &Player,
            &Position,
        )>().iter() {
            to_server.send(WorldToServer::UpdatePlayerPosition {
                receiver_connection_id: broadcast_connection.connection_id,
                connection_id: connection.connection_id,
                x: position.x,
                y: position.y,
            }).unwrap();
        }
    }
}