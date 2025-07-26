use arboard::Clipboard;
use colored::*;
use common::DeviceCodeResponse;
use domain::services::auth::types::GitHubUser;
use domain::services::http_service::HttpService;
use std::io::{self, Write};

/// Display the authentication header and separator
fn display_auth_header() {
    println!("\n{}", "GitHub Device Authentication".bright_white().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
}

/// Display the verification code and copy it to clipboard
fn display_and_copy_code(user_code: &str) {
    println!();
    println!(
        "  {}",
        format!(" Code: {user_code} ")
            .bright_white()
            .bold()
            .on_black()
    );

    // Copy code to clipboard
    match Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(user_code) {
                eprintln!("Failed to copy code to clipboard: {e}");
            } else {
                println!(
                    "  {} {}",
                    "✓".bright_green(),
                    "Code copied to clipboard! You can now paste it on GitHub.".green()
                );
            }
        }
        Err(e) => eprintln!("Failed to access clipboard: {e}"),
    }

    println!();
}

/// Display the verification URL and QR code
fn display_verification_url(verification_uri: &str) {
    println!(
        "{} {}",
        "Verification URL:".bright_white(),
        verification_uri.bright_blue().underline()
    );

    // Display QR code for the verification URL
    println!("\nScan this QR code with your phone:");
    if let Err(e) = qr2term::print_qr(verification_uri) {
        eprintln!("Failed to generate QR code: {e}");
    }
}

/// Prompt user for browser action and handle their choice
fn prompt_browser_action(verification_uri: &str) -> io::Result<()> {
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
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
    );

    if input.trim().to_lowercase() == "y" {
        println!("{} {}", "✓".bright_green(), "Opening browser...".green());
        if let Err(e) = open::that(verification_uri) {
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

    Ok(())
}

/// Main function to orchestrate the OAuth device flow user verification process
pub async fn prompt_user_to_verify(response: &DeviceCodeResponse) {
    // Step 1: Display authentication header
    display_auth_header();

    // Step 2: Display and copy verification code
    display_and_copy_code(&response.user_code);

    // Step 3: Display verification URL and QR code
    display_verification_url(&response.verification_uri);

    // Step 4: Prompt for browser action
    if let Err(e) = prompt_browser_action(&response.verification_uri) {
        eprintln!("Error handling browser prompt: {e}");
    }
}

/// Get user info through the ForkForge API service
///
/// This function now uses the domain service instead of making direct HTTP calls,
/// following the domain-driven design pattern.
pub async fn get_user_info<C>(
    access_token: &str,
    api_service: &HttpService<C>,
) -> Result<GitHubUser, Box<dyn std::error::Error>>
where
    C: domain::services::http::HttpClient,
{
    api_service
        .get_github_user(access_token)
        .await
        .map_err(|e| e.into())
}
