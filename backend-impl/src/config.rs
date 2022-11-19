use std::fs::File;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    pub client_id: String,
    pub issuer: String,
    // "http://localhost:8082/realms/rust-test/protocol/openid-connect/token"
    token_url: Option<String>,
    // "http://localhost:8082/realms/rust-test/protocol/openid-connect/auth"
    auth_url: Option<String>,
}

impl Auth {
    pub fn get_token_url(&self) -> String {
        self.token_url
            .clone()
            .unwrap_or_else(|| format!("{}/protocol/openid-connect/token", self.issuer))
    }
    pub fn get_auth_url(&self) -> String {
        self.auth_url
            .clone()
            .unwrap_or_else(|| format!("{}/protocol/openid-connect/auth", self.issuer))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub auth: Auth,
}
lazy_static! {
    pub static ref CONFIG: Config =
        serde_yaml::from_reader(File::open("config.yaml").expect("Cannot find config.yaml"))
            .expect("Error reading config.yaml");
}
