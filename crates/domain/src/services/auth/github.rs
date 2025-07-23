use crate::errors::DomainError;
use crate::services::auth::types::{
    AuthError, CheckAuthorisationRequest, CheckAuthorisationResponse, DeviceCodeRequest,
    DeviceCodeResponse, GitHubUser,
};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::{sleep, Instant};

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

/// Domain-defined contract for HTTP operations
///
/// This trait defines what the domain needs without knowing HOW it's implemented.
/// Infrastructure provides concrete implementations: `impl HttpClient for ReqwestAdapter`
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError>;
    async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError>;
}

/// Domain service that uses injected HTTP client
///
/// Generic over any `HttpClient` implementation:
/// - Production: `GitHubAuthService<ReqwestAdapter>`
/// - Testing: `GitHubAuthService<MockHttpClient>`
pub struct GitHubAuthService<C: HttpClient> {
    client_id: String,
    http_client: C,
}

impl<C: HttpClient> GitHubAuthService<C> {
    pub fn new(client_id: String, http_client: C) -> Self {
        Self {
            client_id,
            http_client,
        }
    }

    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError> {
        let request = DeviceCodeRequest {
            client_id: self.client_id.clone(),
            scope: "user".to_owned(),
        };

        let body = serde_urlencoded::to_string(&request)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize request: {}", e)))?;

        let response_text = self
            .http_client
            .post_form(GITHUB_DEVICE_CODE_REQUEST_URL, &body)
            .await?;

        serde_json::from_str(&response_text).map_err(|e| {
            DomainError::ExternalService(format!("Failed to parse GitHub response: {}", e))
        })
    }

    pub async fn poll_authorization(
        &self,
        device_code: String,
    ) -> Result<CheckAuthorisationResponse, AuthError> {
        let request = CheckAuthorisationRequest {
            client_id: self.client_id.clone(),
            device_code,
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_owned(),
        };

        let body =
            serde_urlencoded::to_string(&request).map_err(|e| AuthError::InternalServerError {
                debug_info: format!("Failed to serialize request: {}", e),
            })?;

        let start_instant = Instant::now();

        loop {
            if start_instant.elapsed() >= Duration::from_secs(900) {
                return Err(AuthError::UserAuthenticationTimeout);
            }

            sleep(Duration::from_secs(5)).await;

            let response_text = self
                .http_client
                .post_form(GITHUB_CHECK_USER_AUTHORISED_URL, &body)
                .await
                .map_err(|e| AuthError::InternalServerError {
                    debug_info: format!("Failed to send request: {}", e),
                })?;

            if let Ok(error_response) =
                serde_json::from_str::<GitHubDeviceFlowError>(&response_text)
            {
                match error_response.error {
                    GitHubDeviceFlowErrorType::AuthorizationPending => continue,
                    GitHubDeviceFlowErrorType::SlowDown => {
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    GitHubDeviceFlowErrorType::ExpiredToken => {
                        return Err(AuthError::UserAuthenticationTimeout);
                    }
                    GitHubDeviceFlowErrorType::AccessDenied => {
                        return Err(AuthError::UserDeniedAuthentication);
                    }
                    GitHubDeviceFlowErrorType::IncorrectClientCredentials => {
                        return Err(AuthError::ServerConfigurationError {
                            debug_info: "Invalid client credentials".to_string(),
                        });
                    }
                    GitHubDeviceFlowErrorType::IncorrectDeviceCode => {
                        return Err(AuthError::ServerConfigurationError {
                            debug_info: "Incorrect device code".to_string(),
                        });
                    }
                    GitHubDeviceFlowErrorType::DeviceFlowDisabled => {
                        return Err(AuthError::InternalServerError {
                            debug_info: "Device flow disabled".to_string(),
                        });
                    }
                    _ => {
                        return Err(AuthError::InternalServerError {
                            debug_info: format!("Unexpected error: {:?}", error_response.error),
                        });
                    }
                }
            }

            let success_response: CheckAuthorisationResponse = serde_json::from_str(&response_text)
                .map_err(|e| AuthError::InternalServerError {
                    debug_info: format!("Failed to parse success response: {}", e),
                })?;

            return Ok(success_response);
        }
    }

    pub async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError> {
        let response_text = self
            .http_client
            .get_with_auth("https://api.github.com/user", access_token)
            .await?;

        serde_json::from_str(&response_text).map_err(|e| {
            DomainError::ExternalService(format!("Failed to parse GitHub user response: {}", e))
        })
    }
}
