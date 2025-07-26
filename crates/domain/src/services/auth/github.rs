use anyhow::Error;
use chrono::Utc;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::AuthToken;
use crate::repositories::AuthRepository;
use crate::services::auth::types::{AuthError, DeviceCodeResponse};
use crate::services::auth::{ApiToken, AuthenticatedUser, TokenService};

/// Domain-defined contract for device flow authentication
///
/// This trait abstracts the OAuth device flow process without coupling to any specific provider.
/// Infrastructure provides concrete implementations for GitHub, GitLab, etc.
#[async_trait::async_trait]
pub trait DeviceFlowProvider: Send + Sync {
    /// Request a new device code for user authentication
    async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError>;

    /// Poll for user authorization completion
    async fn poll_authorization(&self, device_code: &str) -> Result<String, AuthError>;

    /// Fetch user information using an access token
    async fn get_user(&self, access_token: &str) -> Result<AuthenticatedUser, DomainError>;
}

/// Domain service for authentication operations
///
/// This service orchestrates authentication flows using the injected provider.
/// It's agnostic to the specific OAuth provider (GitHub, GitLab, etc.).
pub struct AuthService<P: DeviceFlowProvider, R: AuthRepository> {
    provider: P,
    auth_repository: R,
}

impl<P: DeviceFlowProvider, R: AuthRepository> AuthService<P, R> {
    pub fn new(provider: P, auth_repository: R) -> Self {
        Self {
            provider,
            auth_repository,
        }
    }

    /// Create a new API token for an authenticated user
    pub async fn create_api_token(
        &self,
        _user: AuthenticatedUser,
        user_id: Uuid,
    ) -> Result<ApiToken, DomainError> {
        // Generate new token
        let token = TokenService::generate_api_token();

        // Hash with user_id as salt
        let token_hash = TokenService::hash_token(&token, &user_id.to_string());

        // Create credentials record
        let credentials = AuthToken {
            id: Uuid::new_v4(),
            user_id,
            token_hash,
            name: todo!(),
            expires_at: None, // No expiry for now
            created_at: Utc::now(),
            last_used_at: None,
        };

        // Store in repository
        self.auth_repository.create(&credentials).await?;

        // Return unhashed token to user
        Ok(ApiToken {
            token,
            expiry: None,
        })
    }

    pub async fn complete_auth_flow(&self, _device_code: &str) -> Result<(), Error> {
        let device_code_response = self.provider.request_device_code().await?;
        // NOTE: We wait here for the user to use the OTP.
        let access_token = self
            .provider
            .poll_authorization(&device_code_response.device_code)
            .await?;
        let _user_details = self.provider.get_user(&access_token).await?;

        // TODO: Need to get or create user_id here before creating token
        // For now, just return Ok - the actual user creation/lookup logic
        // would need to be implemented based on your user management strategy
        Ok(())
    }
}
