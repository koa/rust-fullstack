use std::collections::HashMap;

use actix_4_jwt_auth::{
    biscuit::{Validation, ValidationOptions},
    DecodedInfo, OIDCValidationError, Oidc, OidcBiscuitValidator, OidcConfig,
};
use actix_web::{
    get,
    guard::Post,
    middleware::Logger,
    web::{resource, Data},
    App, HttpServer,
};
use actix_web_prometheus::PrometheusMetricsBuilder;
use actix_web_static_files::ResourceFiles;
use async_graphql::futures_util::future::join_all;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use env_logger::Env;
use log::{error, info, trace};
use prometheus::{histogram_opts, HistogramVec};
use static_files::Resource;
use thiserror::Error;
use tracing_actix_web::TracingLogger;

use backend_impl::{
    config::CONFIG, context::UserInfo, create_anonymous_schema, create_schema,
    AnonymousGraphqlSchema, GraphqlSchema,
};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

async fn graphql(
    context: Data<ApplicationContext>,
    user: Option<DecodedInfo>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    //let user: Option<AuthenticatedUser<UserInfo>> = Some(user);
    trace!("Execute Authenticated: {user:#?}");
    let schema = &context.schema;
    let histogram = context.graphql_request_histogram.clone();
    let request = request.into_inner();
    let found_user = if let Some(DecodedInfo { jwt: _jwt, payload }) = user {
        match serde_json::from_value::<UserInfo>(payload.private.clone()) {
            Ok(user) => Some(user),
            Err(error) => {
                error!("Cannot decode user info: {:#?}: {error}", payload.private);
                None
            }
        }
    } else {
        None
    };
    let timer = histogram
        .with_label_values(&[
            request.operation_name.as_deref().unwrap_or_default(),
            found_user
                .as_ref()
                .map(|u| u.name.as_str())
                .unwrap_or_default(),
        ])
        .start_timer();
    let request = if let Some(user) = found_user {
        request.data(user)
    } else {
        request
    };

    let response = schema.execute(request).await;
    timer.stop_and_record();
    response.into()
}
async fn graphql_anonymous(
    context: Data<ApplicationContext>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    let schema = &context.anonymous_schema;
    let histogram = context.graphql_request_histogram.clone();
    let request = request.into_inner();
    let timer = histogram
        .with_label_values(&[request.operation_name.as_deref().unwrap_or_default(), ""])
        .start_timer();

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
    schema: GraphqlSchema,
    anonymous_schema: AnonymousGraphqlSchema,
}

#[derive(Error, Debug)]
enum BackendError {
    #[error("An IO Error happened {0}")]
    IO(#[from] std::io::Error),
    #[error("An Error from prometheus {0}")]
    Prometheus(#[from] prometheus::Error),
    #[error("An Error from prometheus {0}")]
    ActixWebPrometheus(#[from] actix_web_prometheus::error::Error),
    #[error("Error on OIDC Validation {0}")]
    OidcValidationError(#[from] OIDCValidationError),
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
        .build()?;

    let registry = prometheus.registry.clone();
    registry.register(Box::new(graphql_request_histogram.clone()))?;

    let schema = create_schema();
    let anonymous_schema = create_anonymous_schema();

    let issuer = CONFIG.auth_issuer().to_string();
    info!("Issuer: {issuer}");
    let oidc = Oidc::new(OidcConfig::Issuer(issuer.clone().into())).await?;

    let biscuit_validator = OidcBiscuitValidator {
        options: ValidationOptions {
            issuer: Validation::Validate(issuer),
            ..ValidationOptions::default()
        },
    };

    let data = Data::new(ApplicationContext {
        graphql_request_histogram,
        schema,
        anonymous_schema,
    });
    let main_server = HttpServer::new(move || {
        let resources: HashMap<&str, Resource> = generate();

        App::new()
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .app_data(data.clone())
            .app_data(oidc.clone())
            .service(
                resource("/graphql")
                    .guard(Post())
                    .wrap(biscuit_validator.clone())
                    .to(graphql),
            )
            .service(
                resource("/graphql_anonymous")
                    .guard(Post())
                    .to(graphql_anonymous),
            )
            // workaround for proxy troubles
            .service(
                resource("/graphql/")
                    .guard(Post())
                    .wrap(biscuit_validator.clone())
                    .to(graphql),
            )
            .service(
                resource("/graphql_anonymous/")
                    .guard(Post())
                    .to(graphql_anonymous),
            )
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
