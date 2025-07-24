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