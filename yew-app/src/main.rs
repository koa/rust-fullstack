use yew::start_app;

use crate::app::App;

mod app;
mod error;
mod graphql;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    start_app::<App>();
}
