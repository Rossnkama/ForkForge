use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::Serialize;

use forkforge_config::Config;

#[derive(Serialize)]
struct ApiResponse<T> {
    data: T,
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse { data: "Ok" })
}

async fn new_session() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        data: "Starting session stub",
    })
}

async fn new_snapshot(Path(_id): Path<String>) -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        data: "Starting snapshot stub",
    })
}

async fn stripe_webhook() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse {
        data: "Starting webhook stub",
    })
}

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    let app = Router::new()
        .route("/health", get(health))
        .route("/sessions", post(new_session))
        .route("/snapshots/{:id}", post(new_snapshot))
        .route("/billing/webhook", post(stripe_webhook));

    let addr = format!("{}:{}", config.api_host, config.api_port);
    println!("Server listening on... {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
