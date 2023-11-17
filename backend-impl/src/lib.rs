use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

use crate::config::CONFIG;
use crate::context::UserInfo;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Query;
pub struct QueryAnonymous;

#[Object]
impl QueryAnonymous {
    /// gives the coordinates for authentication
    async fn authentication(&self) -> AuthenticationData {
        AuthenticationData {
            client_id: CONFIG.auth_client_id(),
            auth_url: CONFIG.auth_url(),
            token_url: CONFIG.auth_token_url(),
        }
    }
}
#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, ctx: &Context<'_>, a: i32, b: i32) -> async_graphql::Result<i32> {
        ctx.data::<UserInfo>()?;
        Ok(a + b)
    }
}

#[derive(SimpleObject)]
struct AuthenticationData {
    client_id: &'static str,
    token_url: String,
    auth_url: String,
}

pub type GraphqlSchema = Schema<Query, EmptyMutation, EmptySubscription>;
pub type AnonymousGraphqlSchema = Schema<QueryAnonymous, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> GraphqlSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}
pub fn create_anonymous_schema() -> AnonymousGraphqlSchema {
    Schema::build(QueryAnonymous, EmptyMutation, EmptySubscription).finish()
}

#[cfg(test)]
mod tests {}

pub mod config;
pub mod context {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    pub struct UserInfo {
        pub name: String,
        pub email: Option<String>,
        pub email_verified: Option<bool>,
    }
}
