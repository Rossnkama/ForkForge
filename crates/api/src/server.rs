//! # ForkForge API Server
//!
//! This is the main HTTP API server for ForkForge/Chainbox, built with Axum.
//! It provides REST endpoints for authentication, session management, and billing.
//!
//! ## Architecture
//!
//! The server uses the `ServerInfra` façade from the infra crate to access all
//! infrastructure services (database, external APIs, etc.) while keeping the
//! HTTP layer focused on request/response handling.
//!
//! ## Endpoints
//!
//! - Authentication: GitHub OAuth device flow
//! - Sessions: Fork session management
//! - Snapshots: Time-travel snapshot creation
//! - Billing: Stripe webhook handling

mod github;

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::Serialize;
use std::sync::Arc;

use common::Config;
use domain::{
    repositories::{AuthRepository, UserRepository},
    services::auth::github::AuthService,
};
use github::github_create_user_device_session;
use infra::{GitHubDeviceFlowProvider, ServerInfra};

use crate::github::{check_user_authorised, github_login};

/// Application state shared across all request handlers
///
/// Contains configuration and service instances needed by handlers.
/// Cloned for each request due to Axum's state management.
// TODO: Add some sort of rate limiting to the requests to github.com
#[derive(Clone)]
pub(crate) struct AppState {
    config: Config,
    #[allow(dead_code)]
    infra: Arc<ServerInfra>,
    github_auth_service: Arc<AuthService<GitHubDeviceFlowProvider, AuthRepository>>,
}

#[allow(dead_code)]
impl AppState {
    fn config(&self) -> &Config {
        &self.config
    }
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
    // TODO: Use domain::services::sessions::create_session
    Json(ApiResponse {
        data: "Starting session stub",
    })
}

async fn new_snapshot(Path(_id): Path<String>) -> Json<ApiResponse<&'static str>> {
    // TODO: Use domain::services::snapshots::create_snapshot
    Json(ApiResponse {
        data: "Starting snapshot stub",
    })
}

async fn stripe_webhook() -> Json<ApiResponse<&'static str>> {
    // TODO: Use domain::services::billing::webhooks::process_stripe_webhook
    Json(ApiResponse {
        data: "Starting webhook stub",
    })
}

/// Main entry point for the API server
///
/// Initializes all infrastructure services via `ServerInfra`, sets up
/// domain services, configures routes, and starts the HTTP server.
///
/// ## Initialization Order
///
/// 1. Load configuration from config.toml and environment
/// 2. Initialize infrastructure (database, HTTP clients, Stripe)
/// 3. Create domain services with dependency injection
/// 4. Configure HTTP routes
/// 5. Start server on configured host:port
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize infrastructure
    let infra = Arc::new(
        ServerInfra::new(&config)
            .await
            .expect("Failed to initialize infrastructure"),
    );

    // Create GitHub device flow provider and auth service
    let device_flow_provider = GitHubDeviceFlowProvider::new(
        config
            .github_client_id
            .clone()
            .expect("GitHub client ID not configured"),
        infra.http.clone(),
    );

    let github_auth_service = Arc::new(AuthService::new(
        device_flow_provider,
        todo!("Add the reposity instance"),
    ));

    let state = AppState {
        config: config.clone(),
        infra,
        github_auth_service,
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
        .route("/auth/github-login", get(github_login))
        .route("/health", get(health))
        .route("/sessions", post(new_session))
        .route("/snapshots/{id}", post(new_snapshot))
        .route("/billing/webhook", post(stripe_webhook))
        .with_state(state);

    let addr = format!("{}:{}", config.api_host, config.api_port);
    println!("Server listening on... {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
