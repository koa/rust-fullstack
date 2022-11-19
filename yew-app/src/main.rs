use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::start_app;

mod app;
mod error;
mod graphql;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(Level::Debug));
    start_app::<app::App>();
}
