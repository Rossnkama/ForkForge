use crate::errors::DomainError;
use crate::services::auth::types::{AuthError, DeviceCodeResponse, GitHubUser};

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
    async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError>;
}

/// Domain service for authentication operations
///
/// This service orchestrates authentication flows using the injected provider.
/// It's agnostic to the specific OAuth provider (GitHub, GitLab, etc.).
pub struct AuthService<P: DeviceFlowProvider> {
    provider: P,
}

impl<P: DeviceFlowProvider> AuthService<P> {
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Initiate device flow authentication
    pub async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError> {
        self.provider.request_device_code().await
    }

    /// Wait for user to complete authorization
    pub async fn wait_for_authorization(&self, device_code: &str) -> Result<String, AuthError> {
        self.provider.poll_authorization(device_code).await
    }

    /// Get authenticated user information
    pub async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError> {
        self.provider.get_user(access_token).await
    }
}
