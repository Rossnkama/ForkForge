//! # ForkForge CLI Client
//!
//! Command-line interface for ForkForge/Chainbox, providing local development
//! tools for Solana mainnet forking.
//!
//! ## Architecture
//!
//! The CLI communicates with the ForkForge API server for authentication and
//! session management. It uses the infra crate's `GitHubHttpClient` for GitHub
//! OAuth operations but maintains its own HTTP client for API communication.
//!
//! ## Commands
//!
//! - `login`: Authenticate via GitHub OAuth device flow
//! - `up`: Launch a forked Solana validator (coming soon)

use clap::{Parser, Subcommand};
use common::{CheckUserAuthorisedResponse, DeviceCodeResponse, PollAuthorizationRequest};
use domain::services::auth::internal_api::InternalApiService;
use domain::services::auth::types::GitHubUser;

mod client_config;
mod github;
mod infrastructure;

use client_config::ClientConfig;
use infrastructure::http_client::GitHubHttpClient;

/// ForkForge CLI - Fast Solana mainnet forking for local development
#[derive(Parser)]
#[command(name="forkforge", version, about, long_about = None)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand)]
enum Commands {
    /// Authenticate with GitHub to access ForkForge services
    Login,
    /// Launch a forked Solana validator with configured accounts
    Up,
}

async fn up(_config: ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Implement Up command!");
}

/// Retrieve device code from GitHub through our API
async fn get_device_code(
    config: &ClientConfig,
) -> Result<DeviceCodeResponse, Box<dyn std::error::Error>> {
    let device_code_url = format!("{}/auth/github/device-code", config.api_base_url);

    let device_response = config
        .http_client
        .post(&device_code_url)
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to get device code from {device_code_url}: {e}"))?;

    let status = device_response.status();
    let body = device_response
        .text()
        .await
        .map_err(|e| format!("Failed to read device code response: {e}"))?;

    if !status.is_success() {
        return Err(format!("Device code API error ({status}): {body}").into());
    }

    let device_auth_data: DeviceCodeResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse device code JSON: {e}\nBody: {body}"))?;

    Ok(device_auth_data)
}

/// Poll for user authorization with GitHub
async fn poll_for_authorization(
    config: &ClientConfig,
    device_code: String,
) -> Result<CheckUserAuthorisedResponse, Box<dyn std::error::Error>> {
    let poll_url = format!("{}/auth/github/wait-for-authorization", config.api_base_url);
    let poll_response = config
        .long_poll_client
        .post(&poll_url)
        .json(&PollAuthorizationRequest { device_code })
        .send()
        .await
        .map_err(|e| format!("Failed to poll authorization at {poll_url}: {e}"))?;

    let status = poll_response.status();
    let body = poll_response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    if !status.is_success() {
        return Err(format!("API error ({status}): {body}").into());
    }

    let auth_response: CheckUserAuthorisedResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse auth response JSON: {e}\nBody: {body}"))?;

    Ok(auth_response)
}

/// Handle the GitHub OAuth login flow
///
/// Implements the complete GitHub device flow authentication:
/// 1. Request device code from API server
/// 2. Display verification URL and code to user
/// 3. Poll for authorization completion
/// 4. Retrieve user information
///
/// Uses the infra crate's GitHubHttpClient for HTTP operations,
/// demonstrating proper use of dependency injection.
async fn handle_login(config: ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create domain services with dependency injection
    let http_adapter = GitHubHttpClient::with_default_client();
    let api_service = InternalApiService::new(config.api_base_url.clone(), http_adapter);

    // Step 1: Get device and user verification codes
    let device_auth_data = get_device_code(&config).await?;

    // Step 2: Prompt user to verify
    github::prompt_user_to_verify(&device_auth_data).await;

    // Step 3: Poll for user authorization
    let auth_response = poll_for_authorization(&config, device_auth_data.device_code).await?;

    // Step 4: Get user info using domain service
    let user: GitHubUser = github::get_user_info(&auth_response.access_token, &api_service).await?;

    // Step 5: Write or update the user's entry in the database.
    // TODO: Later, add a new endpoint to securley generate an API token for the user.
    // We will link this with the TUI (or website) later so that the user can manage their keys.

    // TODO: Replace this with something more fancy like loading bars or something.
    println!(
        "Logging in to user {}... who has ID {}",
        user.login, user.id
    );

    Ok(())
}

/// CLI entry point
///
/// Parses command-line arguments and routes to appropriate command handlers.
/// Loads configuration from environment variables (no config file access for
/// security reasons - CLI doesn't have access to server secrets).
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();
    let config = ClientConfig::load()?;

    match cli.command {
        Some(Commands::Up) => {
            up(config).await?;
        }
        Some(Commands::Login) => {
            handle_login(config).await?;
        }
        _ => {
            panic!("Incorrect Command!");
        }
    }

    Ok(())
}
