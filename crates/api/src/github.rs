use common::{
    CheckUserAuthorisedResponse, DeviceCodeResponse, GitHubUser, PollAuthorizationRequest,
};
use domain::services::auth::AuthError;

use axum::{Json, debug_handler, extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

// Wrapper to implement IntoResponse for domain error types
pub(crate) struct ApiError(AuthError);

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        ApiError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.0 {
            AuthError::UserAuthenticationTimeout => StatusCode::REQUEST_TIMEOUT,
            AuthError::UserDeniedAuthentication => StatusCode::UNAUTHORIZED,
            AuthError::ServerConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(serde_json::json!({ "error": self.0.message() })),
        )
            .into_response()
    }
}

#[debug_handler]
pub(crate) async fn check_user_authorised(
    State(state): State<AppState>,
    Json(poll_request): Json<PollAuthorizationRequest>,
) -> Result<Json<CheckUserAuthorisedResponse>, ApiError> {
    let domain_response = state
        .github_auth_service
        .poll_authorization(poll_request.device_code)
        .await?;

    // Convert domain response to common response type
    let response = CheckUserAuthorisedResponse {
        access_token: domain_response.access_token,
        _token_type: domain_response.token_type,
        _scope: domain_response.scope,
    };

    println!("Authentication successful, Token: {:?}", response);
    Ok(Json(response))
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
) -> Result<Json<DeviceCodeResponse>, StatusCode> {
    let domain_response = state
        .github_auth_service
        .request_device_code()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert domain response to common response type
    let response = DeviceCodeResponse {
        device_code: domain_response.device_code,
        user_code: domain_response.user_code,
        verification_uri: domain_response.verification_uri,
        _expires_in: domain_response.expires_in,
        _interval: domain_response.interval,
    };

    Ok(Json(response))
}

#[debug_handler]
pub async fn github_login(
    State(state): State<AppState>,
    Json(access_token): Json<String>,
) -> Result<Json<GitHubUser>, StatusCode> {
    let domain_user = state
        .github_auth_service
        .get_user(&access_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert domain user to common user type
    let user = GitHubUser {
        id: domain_user.id,
        login: domain_user.login,
    };

    Ok(Json(user))
}
