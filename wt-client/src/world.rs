// world.rs
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;

use hecs::World;

use crate::components::*;
use crate::systems::*;
use crate::render::*;
use crate::network::*;

#[wasm_bindgen]
pub struct WorldWrapper {
    world: World,
    context: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl WorldWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WorldWrapper, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id("my_canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

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


        Ok(WorldWrapper { world, context })
    }

    pub fn update(&mut self) -> Result<(), JsValue> {
        update_tick(&mut self.world);
        render(&self.world, &self.context)
    }

    pub fn receive_message(&mut self, data: &[u8]) {
        if data.is_empty() {
            web_sys::console::warn_1(&"Empty datagram received".into());
            return;
        }

        match ServerToClientMessage::from_u8(data[0]) {
            Some(ServerToClientMessage::Tick) => {
                decode_tick_datagram(data, &mut self.world);
            }

            Some(ServerToClientMessage::CreatePlayer) => {
                decode_create_player_datagram(data, &mut self.world);
            }

            Some(ServerToClientMessage::UpdatePlayerPosition) => {
                decode_update_player_position_datagram(data, &mut self.world);
            }

            None => {
                web_sys::console::warn_1(&format!("Unknown message type: {}", data[0]).into());
            }
        }
    }

    pub fn input_click_pressed(&mut self, x: f32, y: f32) -> Vec<u8> {
        return build_input_click_pressed(x, y);
    }
}
