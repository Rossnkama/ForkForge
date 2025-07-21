use clap::{Parser, Subcommand};
use forkforge_config::Config;
use forkforge_models::{CheckUserAuthorisedResponse, DeviceCodeResponse, PollAuthorizationRequest};

mod github;

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

async fn up(_config: Config) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Implement Up command!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Some(Commands::Up) => {
            up(config).await?;
        }
        Some(Commands::Login) => {
            // Step 1: Get device and user verification codes
            // Call our API endpoint instead of GitHub directly
            let client = reqwest::Client::new();
            let device_auth_data: DeviceCodeResponse = client
                .post(format!("{}/auth/github/device-code", config.api_base_url))
                .json(&serde_json::json!({}))
                .send()
                .await
                .map_err(|e| format!("Failed to connect to API: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse API response: {}", e))?;

            // Step 2: Prompt user to verify
            github::prompt_user_to_verify(&device_auth_data).await;

            // Step 3: Poll for user authorization with extended timeout
            // Create a separate client with 15-minute timeout for the long-polling auth endpoint
            let auth_client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(900)) // 15 minutes
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            let auth_response: CheckUserAuthorisedResponse = auth_client
                .post(format!(
                    "{}/auth/github/wait-for-authorization",
                    config.api_base_url
                ))
                .json(&PollAuthorizationRequest {
                    device_code: device_auth_data.device_code,
                })
                .send()
                .await
                .map_err(|e| format!("Failed to connect to API: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse API response: {}", e))?;

            // Step 4: Get user info
            let user = github::get_user_info(&auth_response.access_token).await?;

            // TODO: Initiate DB operations and start stripe work
            println!("Logged in as: {} (ID: {})", user.login, user.id);
            println!("GitHub user ID: {}", user.id);
        }
        _ => {
            panic!("Incorrect Command!");
        }
    }

    Ok(())
}
