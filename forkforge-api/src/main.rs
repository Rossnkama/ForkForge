use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Serialize;

use forkforge_config::Config;
use forkforge_models::{DeviceCodeRequestParams, DeviceCodeResponse};

use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};

const GITHUB_DEVICE_CODE_REQUEST_URL: &str = "https://github.com/login/device/code";

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

/// Requests device and user verification codes from GitHub's OAuth device flow.
///
/// This is step 1 of the GitHub device flow authentication process.
/// The CLI calls this to initiate authentication and get codes that the user
/// will use to authorize the application on GitHub's website.
///
/// # Arguments
///
/// * `device_code_request_params` - Contains:
///   - `client_id`: GitHub OAuth app client ID
///   - `scope`: OAuth scopes string (e.g., "read:user, user:email")
///
/// # Returns
///
/// Returns a `DeviceCodeResponse` containing:
/// - `device_code`: Long code used by the CLI for polling authorization status
/// - `user_code`: Short code displayed to the user to enter on GitHub
/// - `verification_uri`: URL where user enters the code (github.com/login/device)
/// - `expires_in`: How long codes are valid (seconds)
/// - `interval`: How often to poll for authorization (seconds)
///
/// # Errors
///
/// Returns an error if:
/// - Network request fails
/// - GitHub returns an error response
/// - Response parsing fails
#[debug_handler]
async fn github_create_user_device_session(
    State(state): State<AppState>,
) -> Json<DeviceCodeResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    // TODO: 1. Use proper error handling
    let device_code_request_params = DeviceCodeRequestParams {
        client_id: state
            .config
            .github_client_id
            .clone()
            .expect("GitHub client ID not configured"),
        scope: "user".to_owned(),
    };
    let body = serde_urlencoded::to_string(device_code_request_params)
        .expect("Failed to serialize request params");

    let response_headers = state
        .http_client
        .post(GITHUB_DEVICE_CODE_REQUEST_URL)
        .headers(headers)
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    let response: DeviceCodeResponse = response_headers
        .json()
        .await
        .expect("Failed to parse response");

    Json(response)
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
