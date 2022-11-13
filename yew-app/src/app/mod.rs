use yew::{function_component, html};

use crate::app::components::adder::Adder;

mod components;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <h1><Adder/></h1>
    }
}
