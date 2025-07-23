use domain::errors::DomainError;
use domain::services::auth::github::HttpClient;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpClient for ReqwestHttpClient {
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
