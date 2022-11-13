use log::error;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use wasm_bindgen::JsValue;

pub struct JavascriptError {
    original_value: JsValue,
}

impl JavascriptError {
    pub fn new(value: JsValue) -> Self {
        JavascriptError {
            original_value: value,
        }
    }
}

impl From<JsValue> for JavascriptError {
    fn from(value: JsValue) -> Self {
        JavascriptError {
            original_value: value,
        }
    }
}

impl Debug for JavascriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(string) = self.original_value.as_string() {
            f.write_str(&string)?;
        }
        Ok(())
    }
}

impl Display for JavascriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(string) = self.original_value.as_string() {
            f.write_str(&string)?;
        }
        Ok(())
    }
}

impl Error for JavascriptError {}

#[derive(Error, Debug)]
pub enum FrontendError {
    #[error("Generic Javascript error")]
    JSError(#[from] JavascriptError),
    #[error("Cannot find Window reference")]
    WindowMissing,
    #[error("Cannot convert json")]
    SerdeError(#[from] serde_json::Error),
    #[error("Graphql Execution Error")]
    GraphqlError(Vec<graphql_client::Error>),
}
