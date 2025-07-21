use colored::*;
use forkforge_models::{
    CheckUserAuthorisedRequestParams, CheckUserAuthorisedResponse, DeviceCodeResponse, GitHubUser,
};
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::{Instant, sleep};

const GITHUB_CHECK_USER_AUTHORISED_URL: &str = "https://github.com/login/oauth/access_token";

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

#[derive(Debug, Deserialize)]
struct GitHubDeviceFlowError {
    error: GitHubDeviceFlowErrorType,
    #[serde(rename = "error_description")]
    _error_description: String,
    #[serde(rename = "error_uri")]
    _error_uri: String,
}

#[derive(Debug)]
pub enum UserFacingError {
    // Authentication specific errors
    UserAuthenticationTimeout,
    UserDeniedAuthentication,

    // Server/backend errors (should be vague)
    ServerConfigurationError { debug_info: String },
    InternalServerError { debug_info: String },
}

// TODO: Use tracing lib instead of these macros, they'll soon become jarring to manage.
impl UserFacingError {
    fn message(&self) -> String {
        match self {
            UserFacingError::UserAuthenticationTimeout => {
                "Authentication timed out. Please try logging in again.".to_string()
            }
            UserFacingError::UserDeniedAuthentication => {
                "Authentication was denied. Please check your permissions and try again."
                    .to_string()
            }
            UserFacingError::ServerConfigurationError { debug_info } => {
                #[cfg(debug_assertions)]
                {
                    format!("Server configuration error. [DEBUG: {}]", debug_info)
                }
                #[cfg(not(debug_assertions))]
                {
                    "Something went wrong on our end. We're looking into it.".to_string()
                }
            }
            UserFacingError::InternalServerError { debug_info } => {
                #[cfg(debug_assertions)]
                {
                    format!("Internal server error. [DEBUG: {}]", debug_info)
                }
                #[cfg(not(debug_assertions))]
                {
                    "Something went wrong on our end. We're looking into it.".to_string()
                }
            }
        }
    }
}

impl std::fmt::Display for UserFacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for UserFacingError {}

pub async fn prompt_user_to_verify(response: &DeviceCodeResponse) {
    println!("\n{}", "GitHub Device Authentication".bright_white().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    // Highlight the verification code
    println!();
    println!(
        "  {}",
        format!("Code: {}", response.user_code)
            .bright_white()
            .bold()
            .on_blue()
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

pub async fn check_user_authorised(
    check_user_authorised_request_params: CheckUserAuthorisedRequestParams,
) -> Result<CheckUserAuthorisedResponse, Box<dyn std::error::Error>> {
    let http_client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let body = serde_urlencoded::to_string(check_user_authorised_request_params)?;

    let start_instant = Instant::now();

    loop {
        // Check timeout before processing error
        if start_instant.elapsed() >= Duration::from_secs(900) {
            return Err(Box::new(UserFacingError::UserAuthenticationTimeout));
        }

        sleep(Duration::from_secs(5)).await;

        let response_headers = http_client
            .post(GITHUB_CHECK_USER_AUTHORISED_URL)
            .headers(headers.clone())
            .body(body.clone())
            .send()
            .await?;

        let response_text = response_headers.text().await?;

        // Try to parse as error first (most common case during polling)
        if let Ok(error_response) = serde_json::from_str::<GitHubDeviceFlowError>(&response_text) {
            match error_response.error {
                GitHubDeviceFlowErrorType::AuthorizationPending => continue,
                GitHubDeviceFlowErrorType::SlowDown => {
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
                GitHubDeviceFlowErrorType::ExpiredToken => {
                    return Err(Box::new(UserFacingError::UserAuthenticationTimeout));
                }
                GitHubDeviceFlowErrorType::UnsupportedGrantType => {
                    return Err(Box::new(UserFacingError::InternalServerError {
                        debug_info: "Unsupported grant type".to_string(),
                    }));
                }
                GitHubDeviceFlowErrorType::IncorrectClientCredentials => {
                    return Err(Box::new(UserFacingError::ServerConfigurationError {
                        debug_info: "Invalid client credentials such as client_id".to_string(),
                    }));
                }
                GitHubDeviceFlowErrorType::IncorrectDeviceCode => {
                    return Err(Box::new(UserFacingError::ServerConfigurationError {
                        debug_info: "Incorrect Device Code".to_string(),
                    }));
                }
                GitHubDeviceFlowErrorType::AccessDenied => {
                    return Err(Box::new(UserFacingError::UserDeniedAuthentication));
                }
                GitHubDeviceFlowErrorType::DeviceFlowDisabled => {
                    return Err(Box::new(UserFacingError::InternalServerError {
                        debug_info: "Device flow disabled in github app settings".to_string(),
                    }));
                }
            }
        }

        // If not an error, must be success
        let success_response: CheckUserAuthorisedResponse = serde_json::from_str(&response_text)?;
        println!("Authentication successful, Token: {:?}", success_response);
        return Ok(success_response);
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
