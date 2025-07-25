use crate::components::*;

pub fn collision_slide_velocity(
    position: &Position,
    velocity: &Velocity,
    player_collision: &PlayerCollision,
    collision: &Collision,
    iterations: u8,
) -> (f32, f32) {
    let mut result_velocity_x = velocity.x;
    let mut result_velocity_y = velocity.y;

    let mut collided = false;

    for _ in 0..iterations {
        let predicted_x = (position.x + player_collision.offset_x) + result_velocity_x;
        let predicted_y = (position.y + player_collision.offset_y) + result_velocity_y;

        collided = false;
        
        for line in &collision.collision_lines {
            let x1 = line.x1;
            let y1 = line.y1;
            let x2 = line.x2;
            let y2 = line.y2;

            let min_x = x1.min(x2) - player_collision.radius;
            let max_x = x1.max(x2) + player_collision.radius;
            let min_y = y1.min(y2) - player_collision.radius;
            let max_y = y1.max(y2) + player_collision.radius;

            if predicted_x < min_x || predicted_x > max_x || predicted_y < min_y || predicted_y > max_y {
                continue;
            }

            

            let dx = x2 - x1;
            let dy = y2 - y1;
            let line_length_squared = dx * dx + dy * dy;
            if line_length_squared == 0.0 {
                continue;
            }

            

            let t = (((predicted_x - x1) * dx + (predicted_y - y1) * dy) / line_length_squared)
                .clamp(0.0, 1.0);

            let closest_x = x1 + t * dx;
            let closest_y = y1 + t * dy;

            let distance_x = predicted_x - closest_x;
            let distance_y = predicted_y - closest_y;
            let distance_squared = distance_x * distance_x + distance_y * distance_y;

            if distance_squared < player_collision.radius * player_collision.radius {
                collided = true;

                let distance = distance_squared.sqrt();
                let penetration = player_collision.radius - distance;

                let (nx, ny) = if distance != 0.0 {
                    (distance_x / distance, distance_y / distance)
                } else {
                    (0.0, 0.0)
                };

                result_velocity_x += nx * penetration;
                result_velocity_y += ny * penetration;

                let dot = result_velocity_x * nx + result_velocity_y * ny;
                if dot < 0.0 {
                    result_velocity_x -= nx * dot;
                    result_velocity_y -= ny * dot;
                }

                break; // Check again from start
            }
        }

        if !collided {
            break;
        }
    }

    if collided || (result_velocity_x.abs() <= 0.1 && result_velocity_y.abs() <= 0.1) {
        result_velocity_x = 0.0;
        result_velocity_y = 0.0;
    }

    (result_velocity_x, result_velocity_y)
}