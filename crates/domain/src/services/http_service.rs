use crate::errors::DomainError;
use crate::services::auth::types::GitHubUser;
use crate::services::http::HttpClient;

/// Domain service for HTTP operations
///
/// This service provides a generic wrapper for HTTP operations,
/// supporting both internal API calls and external service communication.
pub struct HttpService<C: HttpClient> {
    api_base_url: String,
    http_client: C,
}

impl<C: HttpClient> HttpService<C> {
    pub fn new(api_base_url: String, http_client: C) -> Self {
        Self {
            api_base_url,
            http_client,
        }
    }

    /// Get GitHub user info through the API server
    ///
    /// This calls the API server's /auth/github-login endpoint,
    /// which may perform additional validation or data enrichment.
    pub async fn get_github_user(&self, access_token: &str) -> Result<GitHubUser, DomainError> {
        let url = format!("{}/auth/github-login", self.api_base_url);
        self.http_client.get_json(&url, Some(access_token)).await
    }
}
