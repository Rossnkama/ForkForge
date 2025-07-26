//! # HTTP Client Module
//!
//! This module provides a generic HTTP client implementation that can be used
//! for various HTTP operations including OAuth flows and API communication.
//!
//! ## Security Note
//!
//! This adapter is safe for both server and client use as it doesn't contain
//! any hardcoded secrets. It relies on tokens provided by the caller.

use async_trait::async_trait;
use domain::errors::DomainError;
use domain::services::http::HttpClient as DomainHttpClient;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};

/// Generic HTTP client for various API operations
///
/// This client provides a unified HTTP implementation that can be used
/// for OAuth flows, API communication, and other HTTP operations.
///
/// # Features
///
/// - OAuth device flow support
/// - API data retrieval with authentication
/// - Generic JSON and form-encoded requests
/// - Connection pooling and timeout configuration
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// Creates a new HttpClient with the provided HTTP client
    ///
    /// # Arguments
    ///
    /// * `client` - Pre-configured reqwest Client with desired settings
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new HttpClient with default client configuration
    ///
    /// # Default Settings
    ///
    /// - Connection pool idle timeout: 90 seconds
    /// - Max idle connections per host: 10
    /// - Request timeout: 30 seconds
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be built (should not happen in practice)
    pub fn with_default_client() -> Self {
        let client = Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }
}

impl HttpClient {
    /// Post form-encoded data to a URL
    pub async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
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
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {e}")))
    }

    /// Get data with authentication header
    pub async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json")
            .header("User-Agent", "forkforge-cli")
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {e}")))?;

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
            .map_err(|e| DomainError::ExternalService(format!("Failed to read response: {e}")))
    }
}

/// Implementation for domain HTTP operations
#[async_trait]
impl DomainHttpClient for HttpClient {
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
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to parse response: {e}")))
    }

    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError> {
        self.post_form(url, body).await
    }

    async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError> {
        self.get_with_auth(url, token).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &(impl serde::Serialize + Sync),
    ) -> Result<T, DomainError> {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(DomainError::ExternalService(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to parse response: {e}")))
    }
}
