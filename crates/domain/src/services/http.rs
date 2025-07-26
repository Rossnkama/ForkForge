use crate::errors::DomainError;

/// Domain-defined contract for HTTP operations
///
/// This trait defines the HTTP capabilities required by the domain layer.
/// Infrastructure layer provides concrete implementations.
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    /// Perform a GET request and deserialize JSON response
    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: Option<&str>,
    ) -> Result<T, DomainError>;

    /// Perform a POST request with form-encoded data
    async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError>;

    /// Perform a GET request with Bearer token authentication
    async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError>;

    /// Perform a POST request with JSON data
    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &(impl serde::Serialize + Sync),
    ) -> Result<T, DomainError>;
}
