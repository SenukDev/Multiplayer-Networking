use uuid::Uuid;

#[derive(Debug)]
pub struct Tick {
    pub tick: u64,
}

#[derive(Debug)]
pub struct Player;

#[derive(Debug)]
pub struct Connection {
    pub connection_id: Uuid,
}

#[derive(Debug)]
pub struct State {
    pub state: PlayerState,
}

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct MoveTarget {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct PlayerMove {
    pub move_speed: f32,
    pub move_input_type: MovementType,
    pub timer: u8,
    //pub timer_threshold: u8,
    //pub direction_radius: f32
}

#[derive(Debug)]
pub struct PlayerCollision {
    pub radius: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug)]
pub struct CollisionLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug)]
pub struct Collision {
    pub collision_lines: Vec<CollisionLine>, 
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Idle,
    Move,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementType {
    Target,
    //Direction,
}