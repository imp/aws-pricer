use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use axum::{routing::get, Router};
use serde_json as json;

use credentials::ShuttleSecretsAwsCredentials;

mod credentials;
mod params;
mod pricing;
mod state;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] shuttle_secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let pricing = pricing::AwsPricingClient::new(shuttle_secrets).await;
    let state = state::State::new(pricing).await;
    let router = Router::new()
        .route("/", get(index))
        .route("/env", get(environment))
        .route("/services", get(services))
        .route("/services/:code", get(service))
        .route("/services/:code/:attribute", get(attribute))
        .route("/products/:code", get(products))
        .with_state(Arc::new(state));

    Ok(router.into())
}

#[axum::debug_handler]
async fn index() -> impl IntoResponse {
    Html(include_str!("assets/index.html"))
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
async fn services(State(state): State<Arc<state::State>>) -> Json<HashMap<String, Vec<String>>> {
    let services = state.pricing().services().await;
    Json(services)
}

#[axum::debug_handler]
async fn service(
    State(state): State<Arc<state::State>>,
    Path(code): Path<String>,
) -> Json<Vec<String>> {
    let service = state.pricing().service(code).await;
    Json(service)
}

#[axum::debug_handler]
async fn attribute(
    State(state): State<Arc<state::State>>,
    Path((code, attribute)): Path<(String, String)>,
) -> Json<Vec<String>> {
    let service = state.pricing().attribute(code, attribute).await;
    Json(service)
}

#[axum::debug_handler]
async fn products(
    State(state): State<Arc<state::State>>,
    Path(code): Path<String>,
) -> Json<Vec<json::Value>> {
    let products = state.pricing().products(code).await;
    Json(products)
}
