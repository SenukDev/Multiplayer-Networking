use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use crate::components::*;
use hecs::World;

pub fn render(world: &World, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    // Draw background
    context.set_fill_style(&JsValue::from_str("#000000"));
    context.fill_rect(0.0, 0.0, 512.0, 384.0);
    
    //Draw collision lines
    context.set_stroke_style(&JsValue::from_str("#FFFFFF"));
    for (_, collision) in world.query::<&Collision>().iter() {
        for line in &collision.collision_lines {
            context.begin_path();
            context.move_to(f64::from(line.x1), f64::from(line.y1));
            context.line_to(f64::from(line.x2), f64::from(line.y2));
            context.stroke();
        }
    }

    // Draw player
    context.set_stroke_style(&JsValue::from_str("#FFFFFF"));
    context.set_fill_style(&JsValue::from_str("#FFFFFF"));
    for (_, (
        _,
        position,
        collision
    )) in world.query::<(
        &Player,
        &Position,
        &PlayerCollision
    )>().iter() {
        // Collision circle
        context.begin_path();
        context.ellipse(
            (position.x + collision.offset_x) as f64,
            (position.y + collision.offset_y) as f64,
            collision.radius as f64,
            collision.radius as f64,
            0.0, 0.0, std::f64::consts::PI * 2.0
        )?;
        context.stroke();

        // Player center
        context.begin_path();
        context.ellipse(
            position.x as f64, position.y as f64,
            4.0, 4.0, 0.0, 0.0, std::f64::consts::PI * 2.0
        )?;
        context.fill();
    }

    Ok(())
}
