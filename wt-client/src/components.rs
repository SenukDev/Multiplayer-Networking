use uuid::Uuid;

#[derive(Debug)]
pub struct Tick {
    pub tick: u64,
}

#[derive(Debug)]
pub struct Player;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Connection {
    pub connection_id: Uuid,
}

#[derive(Debug)]
pub struct PlayerCollision {
    pub radius: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}