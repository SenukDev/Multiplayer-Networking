mod components;
mod systems;
mod render;
mod world;
mod network;

pub use world::WorldWrapper;

use std::sync::Once;

static INIT: Once = Once::new();

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    INIT.call_once(|| {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    });
}