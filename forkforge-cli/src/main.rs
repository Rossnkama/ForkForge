use clap::{Parser, Subcommand};
use std::time::Duration;

use forkforge_config::Config;

/// Simple program to greet a person
#[derive(Parser)]
#[command(name="chainbox", version, about, long_about = None)]
struct Cli {
    /// Command you want forkforge to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Up,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();
    let config = Config::load()?;

    if let Some(Commands::Up) = cli.command {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.api_timeout_seconds))
            .build()?;

        let url = format!("http://{}:{}/sessions", config.api_host, config.api_port);
        let response = http_client.post(url).send().await?;

        if response.status().is_success() {
            println!("Session started successfully");
        } else {
            println!("Failed to start session: {}", response.status());
        }
    } else {
        panic!("Incorrect Command!");
    }

    Ok(())
}
