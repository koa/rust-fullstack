use log::error;
use patternfly_yew::prelude::{Form, TextInput, TextInputType};
use wasm_bindgen_futures::spawn_local;
use yew::{
    html::Scope,
    prelude::{html, Component, Context, Html},
};

use crate::graphql::{query, Add};

pub struct Adder {
    a: i64,
    b: i64,
    c: i64,
}

impl Adder {
    fn calculate(&self, scope: Scope<Adder>) {
        let a = self.a;
        let b = self.b;
        spawn_local(async move {
            let result =
                query::<Add, _>(scope.clone(), crate::graphql::add::Variables { a, b }).await;
            match result {
                Ok(value) => scope.send_message(AddMessage::ResultUpdate { a, b, c: value.add }),
                Err(err) => error!("Error on server {err:?}"),
            }
        });
    }
}

pub enum AddMessage {
    AChanged(i64),
    BChanged(i64),
    ResultUpdate { a: i64, b: i64, c: i64 },
}

impl Component for Adder {
    type Message = AddMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Adder { a: 0, b: 0, c: 0 }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddMessage::AChanged(a) => {
                if self.a != a {
                    self.a = a;
                    self.calculate(ctx.link().clone());
                }
                false
            }
            AddMessage::BChanged(b) => {
                if self.b != b {
                    self.b = b;
                    self.calculate(ctx.link().clone());
                }
                false
            }
            AddMessage::ResultUpdate { a, b, c } => {
                if a == self.a && b == self.b && self.c != c {
                    self.c = c;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let a_edit = ctx.link().batch_callback(|value: String| {
            let n: i64 = value.parse().ok()?;
            Some(AddMessage::AChanged(n))
        });
        let b_edit = ctx.link().batch_callback(|value: String| {
            let n: i64 = value.parse().ok()?;
            Some(AddMessage::BChanged(n))
        });
        let result = self.c;
        let a = format!("{}", self.a);
        let b = format!("{}", self.b);
        html! {
            <Form>
                <TextInput r#type={TextInputType::Number} onchange={a_edit} value={a}/>
                <TextInput r#type={TextInputType::Number} onchange={b_edit} value={b}/>
                <span>{result}</span>
            </Form>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.calculate(ctx.link().clone());
        }
    }
}
