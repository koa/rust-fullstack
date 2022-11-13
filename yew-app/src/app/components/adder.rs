// use anyhow::Result;
use graphql_client::{GraphQLQuery, Response};
use patternfly_yew::Form;
use patternfly_yew::TextInput;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Headers, RequestInit};
use web_sys::{Request, RequestMode, Response as Res};
use yew::html::Scope;
use yew::{html, Component, Context, Html};

use crate::error::{FrontendError, JavascriptError};
use crate::graphql::{add, Add};

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
            let c = add(a, b).await.unwrap();
            scope.send_message(AddMessage::ResultUpdate { a, b, c })
        });
    }
}

pub enum AddMessage {
    AChanged(i64),
    BChanged(i64),
    ResultUpdate { a: i64, b: i64, c: i64 },
}

async fn add(a: i64, b: i64) -> Result<i64, FrontendError> {
    let query = serde_json::json!(Add::build_query(add::Variables { a, b }));
    let mut opts = RequestInit::new();
    opts.method("POST");
    let headers = Headers::new().map_err(|e| JavascriptError::new(e))?;
    //headers.set("Accept", "application/json").unwrap();
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| JavascriptError::new(e))?;
    opts.headers(&headers);
    opts.body(Some(&JsValue::from_str(query.to_string().as_str())));
    opts.mode(RequestMode::Cors);
    let url = String::from("/graphql");
    let request =
        Request::new_with_str_and_init(url.as_str(), &opts).map_err(|e| JavascriptError::new(e))?;

    let window = window().map_or_else(|| Err(FrontendError::WindowMissing), |w| Ok(w))?;
    let result = JsFuture::from(window.fetch_with_request(&request)).await;
    let resp_value = result.map_err(|e| JavascriptError::new(e))?;
    let resp: Res = resp_value.dyn_into().map_err(|e| JavascriptError::new(e))?;

    let resp_text = JsFuture::from(resp.text().map_err(|e| JavascriptError::new(e))?)
        .await
        .map_err(|e| JavascriptError::new(e))?
        .as_string();

    if let Some(text) = resp_text {
        let result: Response<add::ResponseData> = serde_json::from_str(&text)?;
        if let Some(content) = result.data {
            return Ok(content.add);
        }
        return Err(FrontendError::GraphqlError(
            result
                .errors
                .into_iter()
                .flat_map(|e| e.into_iter())
                .collect(),
        ));
    }
    Ok(1)
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
        html! {
            <Form>
                <TextInput r#type="number" onchange={a_edit}/>
                <TextInput r#type="number" onchange={b_edit}/>
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
