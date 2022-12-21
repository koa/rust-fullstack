use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

use crate::config::CONFIG;
use crate::context::UserInfo;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Query;

#[Object]
impl Query {
    /// gives the coordinates for authentication
    async fn authentication(&self) -> AuthenticationData {
        AuthenticationData {
            client_id: CONFIG.auth_client_id(),
            auth_url: CONFIG.auth_url(),
            token_url: CONFIG.auth_token_url(),
        }
    }
    /// Returns the sum of a and b
    async fn add(&self, ctx: &Context<'_>, a: i32, b: i32) -> async_graphql::Result<i32> {
        ctx.data::<UserInfo>()?;
        Ok(a + b)
    }
}

fn empty() {}

#[derive(SimpleObject)]
struct AuthenticationData {
    client_id: &'static str,
    token_url: String,
    auth_url: String,
}

pub type GraphqlSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> GraphqlSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {}

pub mod config;
pub mod context {
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
    pub struct UserInfo {
        pub iss: String,
        pub sub: String,
        pub aud: String,
        pub name: String,
        pub email: Option<String>,
        pub email_verified: Option<bool>,
    }
}
