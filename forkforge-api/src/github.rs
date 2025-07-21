use forkforge_models::{
    CheckUserAuthorisedRequestParams, CheckUserAuthorisedResponse, DeviceCodeRequestParams,
    DeviceCodeResponse, PollAuthorizationRequest,
};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::{Instant, sleep};

use axum::{Json, debug_handler, extract::State};

use crate::AppState;

const GITHUB_CHECK_USER_AUTHORISED_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_DEVICE_CODE_REQUEST_URL: &str = "https://github.com/login/device/code";

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

// TODO: Check that this is server side safe
#[debug_handler]
pub(crate) async fn check_user_authorised(
    State(state): State<AppState>,
    Json(poll_request): Json<PollAuthorizationRequest>,
) -> Json<CheckUserAuthorisedResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    // Build the full request params with server-side client_id
    let check_user_authorised_request_params = CheckUserAuthorisedRequestParams {
        client_id: state.config.github_client_id.clone().unwrap_or_else(|| {
            panic!(
                "{}",
                UserFacingError::ServerConfigurationError {
                    debug_info: "GitHub client ID not configured".to_string()
                }
            )
        }),
        device_code: poll_request.device_code,
        grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_owned(),
    };

    let body =
        serde_urlencoded::to_string(check_user_authorised_request_params).unwrap_or_else(|e| {
            panic!(
                "{}",
                UserFacingError::InternalServerError {
                    debug_info: format!("Failed to serialize request params: {}", e)
                }
            )
        });

    let start_instant = Instant::now();

    loop {
        // Check timeout before processing error
        if start_instant.elapsed() >= Duration::from_secs(900) {
            panic!("{}", UserFacingError::UserAuthenticationTimeout);
        }

        sleep(Duration::from_secs(5)).await;

        let response_headers = state
            .http_client
            .post(GITHUB_CHECK_USER_AUTHORISED_URL)
            .headers(headers.clone())
            .body(body.clone())
            .send()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "{}",
                    UserFacingError::InternalServerError {
                        debug_info: format!("Failed to send request to GitHub: {}", e)
                    }
                )
            });

        let response_text = response_headers.text().await.unwrap_or_else(|e| {
            panic!(
                "{}",
                UserFacingError::InternalServerError {
                    debug_info: format!("Failed to get response text: {}", e)
                }
            )
        });

        // Try to parse as error first (most common case during polling)
        if let Ok(error_response) = serde_json::from_str::<GitHubDeviceFlowError>(&response_text) {
            match error_response.error {
                GitHubDeviceFlowErrorType::AuthorizationPending => continue,
                GitHubDeviceFlowErrorType::SlowDown => {
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
                GitHubDeviceFlowErrorType::ExpiredToken => {
                    panic!("{}", UserFacingError::UserAuthenticationTimeout);
                }
                GitHubDeviceFlowErrorType::UnsupportedGrantType => {
                    panic!(
                        "{}",
                        UserFacingError::InternalServerError {
                            debug_info: "Unsupported grant type".to_string(),
                        }
                    );
                }
                GitHubDeviceFlowErrorType::IncorrectClientCredentials => {
                    panic!(
                        "{}",
                        UserFacingError::ServerConfigurationError {
                            debug_info: "Invalid client credentials such as client_id".to_string(),
                        }
                    );
                }
                GitHubDeviceFlowErrorType::IncorrectDeviceCode => {
                    panic!(
                        "{}",
                        UserFacingError::ServerConfigurationError {
                            debug_info: "Incorrect Device Code".to_string(),
                        }
                    );
                }
                GitHubDeviceFlowErrorType::AccessDenied => {
                    panic!("{}", UserFacingError::UserDeniedAuthentication);
                }
                GitHubDeviceFlowErrorType::DeviceFlowDisabled => {
                    panic!(
                        "{}",
                        UserFacingError::InternalServerError {
                            debug_info: "Device flow disabled in github app settings".to_string(),
                        }
                    );
                }
            }
        }

        // If not an error, must be success
        let success_response: CheckUserAuthorisedResponse = serde_json::from_str(&response_text)
            .unwrap_or_else(|e| {
                panic!(
                    "{}",
                    UserFacingError::InternalServerError {
                        debug_info: format!("Failed to parse success response: {}", e)
                    }
                )
            });
        println!("Authentication successful, Token: {:?}", success_response);
        return Json(success_response);
    }
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
pub(crate) async fn github_create_user_device_session(
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
