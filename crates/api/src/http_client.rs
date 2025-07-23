use domain::errors::DomainError;
use domain::services::auth::github::HttpClient;
use reqwest::header::{HeaderMap, HeaderValue};

/// Adapter that implements domain's HttpClient trait using reqwest
///
/// This is the infrastructure layer's concrete implementation.
/// Usage:
///
/// ```rust
/// let adapter = ReqwestAdapter::new(reqwest::Client::new());
/// ```
///
/// Then inject:
///
/// ```rust
/// GitHubAuthService::new(client_id, adapter);
/// ```
///
pub struct ReqwestAdapter {
    client: reqwest::Client,
}

impl ReqwestAdapter {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestAdapter {
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {}", e)))?;

        response
            .text()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {}", e)))
    }

    async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "forkforge-cli")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {}", e)))?;

        response
            .text()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {}", e)))
    }
}
