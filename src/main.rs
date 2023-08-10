use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json as json;

use credentials::ShuttleSecretsAwsCredentials;

mod credentials;
mod gql;
mod params;
mod pricing;
mod state;
mod types;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] shuttle_secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let pricing = pricing::AwsPricingClient::new(shuttle_secrets).await;
    let state = state::State::new(pricing).await;
    let state = Arc::new(state);
    let load = state.clone();
    tokio::spawn(async move { load.load_services().await });
    let schema = gql::schema(state.clone());
    let router = Router::new()
        .route("/", get(index))
        .route(
            "/graphql",
            get(gql::graphiql)
                .post(gql::graphql)
                .layer(Extension(schema)),
        )
        .route("/codes", get(codes))
        .route("/stats", get(stats))
        .route("/env", get(environment))
        .route("/services", get(services))
        .route("/services/:code", get(service))
        .route("/services/:code/:attribute", get(attribute))
        .route("/products/:code", get(products))
        .with_state(state);

    Ok(router.into())
}

#[axum::debug_handler]
async fn index() -> impl IntoResponse {
    Html(include_str!("assets/index.html"))
}

#[axum::debug_handler]
async fn stats(State(state): State<Arc<state::State>>) -> Json<json::Value> {
    Json(state.load_duration().await)
}

#[axum::debug_handler]
async fn codes(State(state): State<Arc<state::State>>) -> Json<json::Value> {
    Json(state.codes().await)
}

#[axum::debug_handler]
async fn environment(Query(params): Query<params::Params>) -> Response {
    let env = env::vars().collect::<HashMap<String, String>>();
    if params.is_html() {
        format!("{env:#?}").into_response()
    } else {
        Json(env).into_response()
    }
}

#[axum::debug_handler]
async fn services(State(state): State<Arc<state::State>>) -> Json<Vec<types::Service>> {
    let services = state.services().await;
    Json(services)
}

#[axum::debug_handler]
async fn service(
    State(state): State<Arc<state::State>>,
    Path(code): Path<String>,
) -> Json<Option<types::Service>> {
    let service = state.service(code).await;
    Json(service)
}

#[axum::debug_handler]
async fn attribute(
    State(state): State<Arc<state::State>>,
    Path((code, attribute)): Path<(String, String)>,
) -> Json<types::Attribute> {
    let attribute = state.attribute(code, attribute).await;
    Json(attribute)
}

#[axum::debug_handler]
async fn products(
    State(state): State<Arc<state::State>>,
    Path(code): Path<String>,
    Query(attributes): Query<HashMap<String, String>>,
) -> Json<Vec<json::Value>> {
    let products = state.products(code, attributes).await;
    Json(products)
}
