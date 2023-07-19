use std::collections::HashMap;
use std::env;

use axum::Json;
use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn environment() -> Json<HashMap<String, String>> {
    let env = env::vars().collect();
    Json(env)
}

async fn secrets(secrets: shuttle_secrets::SecretStore) -> Json<HashMap<String, String>> {
    let secrets = secrets.into_iter().collect();
    Json(secrets)
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_secrets::Secrets] shuttle_secrets: shuttle_secrets::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/env", get(environment))
        .route("/secrets", get(|| async { secrets(shuttle_secrets).await }));

    Ok(router.into())
}
