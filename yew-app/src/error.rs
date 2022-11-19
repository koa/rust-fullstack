use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use log::error;
use reqwest::header::InvalidHeaderValue;
use thiserror::Error;
use wasm_bindgen::JsValue;

pub struct JavascriptError {
    original_value: JsValue,
}

impl JavascriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(string) = self.original_value.as_string() {
            f.write_str(&string)?;
        }
        Ok(())
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
        Self::fmt(self, f)
    }
}

impl Display for JavascriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Self::fmt(self, f)
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
    #[error("Error on http request")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid http header")]
    InvalidHeaderError(#[from] InvalidHeaderValue),
}
