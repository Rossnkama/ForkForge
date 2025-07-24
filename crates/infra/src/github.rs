//! # GitHub HTTP Adapter Module
//!
//! This module provides HTTP client implementations for GitHub OAuth and API operations.
//! It implements domain-defined HTTP client traits to enable GitHub authentication
//! and user data retrieval.
//!
//! ## Security Note
//!
//! This adapter is safe for both server and client use as it doesn't contain
//! any hardcoded secrets. It relies on OAuth tokens provided by the caller.

use async_trait::async_trait;
use domain::errors::DomainError;
use domain::services::auth::internal_api::HttpClient as ForkForgeHttpClient;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};

/// Unified HTTP client for both GitHub and ForkForge API operations
///
/// This client consolidates the HTTP implementations from both
/// the API and CLI crates, providing a single source of truth for HTTP operations.
/// It implements multiple domain HTTP client traits to support various use cases.
///
/// # Features
///
/// - GitHub OAuth device flow support
/// - GitHub API user data retrieval
/// - ForkForge internal API communication
/// - Connection pooling and timeout configuration
#[derive(Clone)]
pub struct GitHubHttpClient {
    client: Client,
}

impl GitHubHttpClient {
    /// Creates a new GitHubHttpClient with the provided HTTP client
    ///
    /// # Arguments
    ///
    /// * `client` - Pre-configured reqwest Client with desired settings
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new GitHubHttpClient with default client configuration
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

impl GitHubHttpClient {
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
            .header("Accept", "application/vnd.github+json")
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

/// Implementation for ForkForge API HTTP operations
#[async_trait]
impl ForkForgeHttpClient for GitHubHttpClient {
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
}
