use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use axum::extract::Path;
use axum::extract::State;
use axum::Json;
use axum::{routing::get, Router};

use credentials::ShuttleSecretsAwsCredentials;

mod credentials;
mod pricing;
mod state;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] shuttle_secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let pricing = pricing::AwsPricingClient::new(shuttle_secrets).await;
    let state = state::State::new(pricing).await;
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/env", get(environment))
        .route("/services", get(services))
        .route("/services/:code", get(service))
        .route("/services/:code/:attribute", get(attribute))
        .with_state(Arc::new(state));

    Ok(router.into())
}

async fn environment() -> Json<HashMap<String, String>> {
    let env = env::vars().collect();
    Json(env)
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
