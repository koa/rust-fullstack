use graphql_client::reqwest::post_graphql;
use graphql_client::GraphQLQuery;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, AUTHORIZATION};
use yew::html::Scope;
use yew::{Callback, Component};
use yew_oauth2::context::OAuth2Context::Authenticated;
use yew_oauth2::context::{Authentication, OAuth2Context};

use crate::error::FrontendError;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "./graphql/schema.graphql",
    query_path = "./graphql/add.graphql",
    response_derives = "Debug"
)]
pub struct Add;

// ---------------------------------------------------
// helper methods

lazy_static! {
    static ref GRAPHQL_URL: String = format!("{}/graphql", host());
}

pub fn host() -> String {
    let location = web_sys::window().unwrap().location();
    let host = location.host().unwrap();
    let protocol = location.protocol().unwrap();
    format!("{protocol}//{host}")
}

/// Send Graphql-Query to server
pub async fn query<Q: GraphQLQuery, S: Component>(
    scope: Scope<S>,
    request: Q::Variables,
) -> Result<Q::ResponseData, FrontendError> {
    let credentials = scope
        .context::<OAuth2Context>(Callback::noop())
        .map(|r| r.0);

    let mut headers = HeaderMap::new();
    if let Some(Authenticated(Authentication { access_token, .. })) = credentials.as_ref() {
        headers.insert(AUTHORIZATION, format!("Bearer {access_token}").parse()?);
    }
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let response = post_graphql::<Q, _>(&client, GRAPHQL_URL.as_str(), request).await?;
    if let Some(data) = response.data {
        Ok(data)
    } else {
        Err(FrontendError::GraphqlError(
            response.errors.unwrap_or_default(),
        ))
    }
}
