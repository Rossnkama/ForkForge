use common::{
    CheckUserAuthorisedResponse, DeviceCodeResponse, GitHubUser, PollAuthorizationRequest,
};
use domain::services::auth::types::AuthError;

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
    let access_token = state
        .github_auth_service
        .wait_for_authorization(&poll_request.device_code)
        .await?;

    // Create response with the access token
    let response = CheckUserAuthorisedResponse {
        access_token,
        _token_type: "bearer".to_string(),
        _scope: "user".to_string(),
    };

    println!("Authentication successful, Token: {response:?}");
    Ok(Json(response))
}

/// HTTP adapter for GitHub device flow initiation.
///
/// This handler demonstrates the adapter pattern - translating between:
/// - **HTTP Layer**: Axum's State extractor, JSON responses, HTTP status codes
/// - **Domain Layer**: Pure business logic with no web framework dependencies
///
/// # Architecture Pattern
///
/// ```text
/// CLI Request → Axum Handler → Domain Service → GitHub API
///                    ↓               ↓
///              (thin adapter)   (business logic)
/// ```
///
/// The handler is intentionally thin - it only:
/// 1. Extracts the domain service from Axum state
/// 2. Calls the domain method (no parameters needed for device flow)
/// 3. Maps the domain response to HTTP JSON response
/// 4. Converts domain errors to HTTP status codes
///
/// # Benefits
///
/// - **Framework Independence**: Could swap Axum for Actix without touching domain
/// - **Testability**: Domain logic testable without spinning up HTTP server
/// - **Single Responsibility**: HTTP concerns stay in API layer only
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
