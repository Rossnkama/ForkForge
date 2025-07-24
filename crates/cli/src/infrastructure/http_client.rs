use domain::errors::DomainError;
use domain::services::auth::github::HttpClient as GitHubHttpClient;
use domain::services::auth::internal_api::HttpClient as ForkForgeHttpClient;
use reqwest::Client;

/// Adapter that implements domain HTTP traits using reqwest
///
/// This allows the CLI to use domain services while keeping
/// the domain layer independent of specific HTTP implementations.
pub struct ReqwestAdapter {
    client: Client,
}

impl ReqwestAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

/// Implementation for ForkForge API HTTP operations
#[async_trait::async_trait]
impl ForkForgeHttpClient for ReqwestAdapter {
    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: Option<&str>,
    ) -> Result<T, DomainError> {
        let mut request = self.client.get(url);

        if let Some(body_content) = body {
            request = request.json(&body_content);
        }

        let response = request
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to parse response: {}", e)))
    }
}

/// Implementation for GitHub API HTTP operations
#[async_trait::async_trait]
impl GitHubHttpClient for ReqwestAdapter {
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .post(url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {}", e)))
    }

    async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "ForkForge-CLI")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {}", e)))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(DomainError::Unauthorized(
                "Invalid access token".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {}", e)))
    }
}
