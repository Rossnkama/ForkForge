use crate::errors::DomainError;
use crate::services::auth::types::GitHubUser;

/// Domain service for interacting with the ForkForge API server
///
/// This service handles communication with our internal API server,
/// as opposed to external services like GitHub's API.
pub struct InternalApiService<C: HttpClient> {
    api_base_url: String,
    http_client: C,
}

/// Domain-defined contract for HTTP operations
///
/// This trait defines what the domain needs for API communication.
/// CLI provides concrete implementation.
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: Option<&str>,
    ) -> Result<T, DomainError>;
}

impl<C: HttpClient> InternalApiService<C> {
    pub fn new(api_base_url: String, http_client: C) -> Self {
        Self {
            api_base_url,
            http_client,
        }
    }

    /// Get GitHub user info through our API server
    ///
    /// This calls our API server's /auth/github-login endpoint,
    /// which may perform additional validation or data enrichment
    /// beyond what GitHub's API provides directly.
    pub async fn get_github_user(&self, access_token: &str) -> Result<GitHubUser, DomainError> {
        let url = format!("{}/auth/github-login", self.api_base_url);
        self.http_client.get_json(&url, Some(access_token)).await
    }
}
