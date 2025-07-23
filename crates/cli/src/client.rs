use clap::{Parser, Subcommand};
use common::{
    CheckUserAuthorisedResponse, DeviceCodeResponse, GitHubUser, PollAuthorizationRequest,
};

mod client_config;
mod github;

use client_config::ClientConfig;

/// Simple program to greet a person
#[derive(Parser)]
#[command(name="forkforge", version, about, long_about = None)]
struct Cli {
    /// Command you want forkforge to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Login,
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
        .map_err(|e| format!("Failed to get device code from {}: {}", device_code_url, e))?;

    let status = device_response.status();
    let body = device_response
        .text()
        .await
        .map_err(|e| format!("Failed to read device code response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Device code API error ({}): {}", status, body).into());
    }

    let device_auth_data: DeviceCodeResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse device code JSON: {}\nBody: {}", e, body))?;

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
        .map_err(|e| format!("Failed to poll authorization at {}: {}", poll_url, e))?;

    let status = poll_response.status();
    let body = poll_response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if !status.is_success() {
        return Err(format!("API error ({}): {}", status, body).into());
    }

    let auth_response: CheckUserAuthorisedResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse auth response JSON: {}\nBody: {}", e, body))?;

    Ok(auth_response)
}

/// Handle the GitHub OAuth login flow
async fn handle_login(config: ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Get device and user verification codes
    let device_auth_data = get_device_code(&config).await?;

    // Step 2: Prompt user to verify
    github::prompt_user_to_verify(&device_auth_data).await;

    // Step 3: Poll for user authorization
    let auth_response = poll_for_authorization(&config, device_auth_data.device_code).await?;

    // Step 4: Get user info, Github ID and Username
    let user: GitHubUser = github::get_user_info(&auth_response.access_token, &config).await?;

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
