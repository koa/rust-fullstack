use std::collections::HashMap;

use actix_4_jwt_auth::{AuthenticatedUser, OIDCValidator, OIDCValidatorConfig};
use actix_web::guard::Post;
use actix_web::web::{resource, Data};
use actix_web::{get, App, HttpServer};
use actix_web_prometheus::PrometheusMetricsBuilder;
use actix_web_static_files::ResourceFiles;
use async_graphql::{futures_util::future::join_all, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use biscuit::ValidationOptions;
use env_logger::Env;
use prometheus::{histogram_opts, HistogramVec};
use static_files::Resource;
use thiserror::Error;

use backend_impl::config::CONFIG;
use backend_impl::context::UserInfo;
use backend_impl::create_schema;
use backend_impl::Query;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn graphql(
    context: Data<ApplicationContext>,
    user: Option<AuthenticatedUser<UserInfo>>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    let schema = &context.schema;
    let histogram = context.graphql_request_histogram.clone();
    let request = request.into_inner();
    let timer = histogram
        .with_label_values(&[
            request.operation_name.as_deref().unwrap_or_default(),
            user.as_ref()
                .map(|u| u.claims.name.as_str())
                .unwrap_or_default(),
        ])
        .start_timer();
    let request = if let Some(AuthenticatedUser { jwt: _, claims }) = user {
        request.data(claims)
    } else {
        request
    };

    let response = schema.execute(request).await;
    timer.stop_and_record();
    response.into()
}

#[get("/health")]
async fn health() -> &'static str {
    "Ok"
}

#[derive(Clone)]
struct ApplicationContext {
    graphql_request_histogram: HistogramVec,
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

#[derive(Error, Debug)]
enum BackendError {
    #[error("An IO Error happened")]
    IO(#[from] std::io::Error),
    #[error("An Error from prometheus")]
    Prometheus(#[from] prometheus::Error),
}

#[actix_web::main]
async fn main() -> Result<(), BackendError> {
    env_logger::init_from_env(Env::default().filter_or("LOG_LEVEL", "debug"));

    let bind_addr = CONFIG.server_bind_address();
    let api_port = CONFIG.server_port();
    let mgmt_port = CONFIG.server_mgmt_port();

    let mut labels = HashMap::new();
    labels.insert("server".to_string(), "api".to_string());

    let graphql_request_histogram = HistogramVec::new(
        histogram_opts!("graphql_request", "Measure graphql queries"),
        &["name", "user"],
    )?;
    let prometheus = PrometheusMetricsBuilder::new("")
        .const_labels(labels)
        .build()
        .unwrap();

    let registry = prometheus.registry.clone();
    registry.register(Box::new(graphql_request_histogram.clone()))?;

    let schema = create_schema();

    let validation_options = ValidationOptions::default();

    let issuer = CONFIG.auth_issuer();
    let created_validator = OIDCValidator::new_from_issuer(issuer.to_string(), validation_options)
        .await
        .unwrap();

    let validator_config = OIDCValidatorConfig {
        issuer: issuer.to_string(),
        validator: created_validator,
    };

    let data = Data::new(ApplicationContext {
        graphql_request_histogram,
        schema,
    });
    let main_server = HttpServer::new(move || {
        let resources: HashMap<&str, Resource> = generate();

        App::new()
            .wrap(prometheus.clone())
            .app_data(data.clone())
            //.app_data(schema.clone())
            .app_data(validator_config.clone())
            .service(resource("/graphql").guard(Post()).to(graphql))
            // workaround for proxy troubles
            .service(resource("/graphql/").guard(Post()).to(graphql))
            .service(ResourceFiles::new("/", resources).resolve_not_found_to_root())
    })
    .bind((bind_addr, api_port))?
    .run();
    let mut labels = HashMap::new();
    labels.insert("server".to_string(), "mgmt".to_string());

    let prometheus = PrometheusMetricsBuilder::new("")
        .const_labels(labels)
        .registry(registry)
        .endpoint("/metrics")
        .build()
        .unwrap();
    let mgmt_server = HttpServer::new(move || App::new().wrap(prometheus.clone()).service(health))
        .bind((bind_addr, mgmt_port))?
        .workers(2)
        .run();
    if let Some(e) = join_all(vec![main_server, mgmt_server])
        .await
        .into_iter()
        .flat_map(|r| r.err())
        .next()
    {
        return Err(e.into());
    }
    Ok(())
}
