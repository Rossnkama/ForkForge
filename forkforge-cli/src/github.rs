use colored::*;
use forkforge_models::{DeviceCodeResponse, GitHubUser};
use reqwest::Client;
use serde::Deserialize;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GitHubDeviceFlowErrorType {
    AuthorizationPending,
    SlowDown,
    ExpiredToken,
    UnsupportedGrantType,
    IncorrectClientCredentials,
    IncorrectDeviceCode,
    AccessDenied,
    DeviceFlowDisabled,
}

pub async fn prompt_user_to_verify(response: &DeviceCodeResponse) {
    println!("\n{}", "GitHub Device Authentication".bright_white().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    // Highlight the verification code
    println!();
    println!(
        "  {}",
        format!(" Code: {} ", response.user_code)
            .bright_white()
            .bold()
            .on_black()
    );
    println!();

    println!(
        "{} {}",
        "Verification URL:".bright_white(),
        response.verification_uri.bright_blue().underline()
    );

    // Display QR code for the verification URL
    println!("\nScan this QR code with your phone:");
    if let Err(e) = qr2term::print_qr(&response.verification_uri) {
        eprintln!("Failed to generate QR code: {}", e);
    }

    // Enhanced prompt with color coding
    println!(
        "\n{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
    );
    println!(
        "{}",
        "Would you like to open the browser automatically?"
            .bright_white()
            .bold()
    );
    println!();
    println!(
        "  {} {} {}",
        "[Y]".bright_green().bold(),
        "→".bright_cyan(),
        "Open browser and continue".green()
    );
    println!(
        "  {} {} {}",
        "[N]".bright_red().bold(),
        "→".bright_cyan(),
        "Skip and enter code manually".red()
    );
    println!();
    print!(
        "{} {} ",
        "Choose:".bright_white().bold(),
        "(y/n)".bright_yellow()
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
    );

    if input.trim().to_lowercase() == "y" {
        println!("{} {}", "✓".bright_green(), "Opening browser...".green());
        if let Err(e) = open::that(&response.verification_uri) {
            eprintln!("{} Failed to open browser: {}", "✗".bright_red(), e);
            println!(
                "\n{}",
                "Please manually navigate to the URL above and enter your verification code."
                    .yellow()
            );
        }
    } else {
        println!(
            "{} {}",
            "→".bright_yellow(),
            "Please manually navigate to the URL above and enter your verification code.".yellow()
        );
    }
}

pub async fn get_user_info(access_token: &str) -> Result<GitHubUser, Box<dyn std::error::Error>> {
    let client = Client::new();
    let user_response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "forkforge-cli")
        .send()
        .await?;

    let user: GitHubUser = user_response.json().await?;
    Ok(user)
}
