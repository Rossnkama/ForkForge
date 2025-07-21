mod github;

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use reqwest::Client;
use serde::Serialize;

use forkforge_config::Config;
use github::github_create_user_device_session;

use crate::github::check_user_authorised;

// TODO: Add some sort of rate limiting to the requests to github.com
#[derive(Clone)]
pub(crate) struct AppState {
    config: Config,
    http_client: Client,
    // Future fields can be added here:
    // db_pool: sqlx::PgPool,
    // redis_client: redis::Client,
    // etc.
}

// TODO: We're gonna start validating incoming requests
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

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Create a persistent HTTP client with connection pooling
    //
    // - pool_max_idle_per_host: Max idle connections per unique host (e.g., github.com:443)
    //   Keeps up to 10 TCP connections open to each host, avoiding TLS handshakes on reuse
    // - pool_idle_timeout: How long to keep idle connections alive (90s)
    // - timeout: Max time for a complete request/response cycle (30s)
    let http_client = Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    let state = AppState {
        config: config.clone(),
        http_client,
    };

    let app = Router::new()
        // Authentication
        .route(
            "/auth/github/device-code",
            post(github_create_user_device_session),
        )
        .route(
            "/auth/github/wait-for-authorization",
            post(check_user_authorised),
        )
        .route("/auth/github-login/{:access_token}", todo!())
        .route("/health", get(health))
        .route("/sessions", post(new_session))
        .route("/snapshots/{:id}", post(new_snapshot))
        .route("/billing/webhook", post(stripe_webhook))
        .with_state(state);

    let addr = format!("{}:{}", config.api_host, config.api_port);
    println!("Server listening on... {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
