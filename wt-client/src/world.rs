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


        Ok(WorldWrapper { world, context })
    }

    pub fn update(&mut self) -> Result<(), JsValue> {
        update_tick(&mut self.world);
        render(&self.world, &self.context)
    }

    pub fn receive_datagram(&mut self, data: &[u8]) {
        if data.is_empty() {
            web_sys::console::warn_1(&"Empty datagram received".into());
            return;
        }

        match MessageType::from_u8(data[0]) {
            Some(MessageType::Tick) => {
                decode_tick_datagram(data, &mut self.world);
            }

            Some(MessageType::CreatePlayer) => {
                decode_create_player_datagram(data, &mut self.world);
            }

            None => {
                web_sys::console::warn_1(&format!("Unknown message type: {}", data[0]).into());
            }
        }
    }
}
