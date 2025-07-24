//! # Infrastructure Layer
//!
//! This crate provides all infrastructure implementations for the ForkForge/Chainbox project.
//! It implements the interfaces defined in the domain layer, following the Dependency Inversion Principle.
//!
//! ## Architecture
//!
//! The infrastructure is split into two main façades:
//!
//! - `ServerInfra`: Contains all services including those with sensitive credentials (database, Stripe API keys)
//! - `ClientInfra`: Contains only client-safe services without any secrets
//!
//! ## Modules
//!
//! - `db`: SQLite/SQLx database implementations of domain repository traits
//! - `github`: HTTP client adapter for GitHub OAuth and API operations
//! - `stripe`: Stripe SDK integration for billing operations
//! - `helius`: Placeholder for future Helius RPC integration

pub mod db;
pub mod github;
pub mod github_device_flow;
pub mod helius;
pub mod stripe;

pub use db::{DbRepo, MIGRATOR};
pub use github::GitHubHttpClient;
pub use github_device_flow::GitHubDeviceFlowProvider;
pub use stripe::{StripeSdk, StripeWebhookHandler};

use domain::errors::DomainError;

/// Server-side infrastructure containing sensitive services
///
/// This struct aggregates all infrastructure services needed by the API server,
/// including those that contain sensitive credentials like database connection
/// strings and Stripe API keys.
///
/// # Security Warning
///
/// This struct should NEVER be used in client/CLI code as it would embed
/// secrets into the client binary. Use `ClientInfra` instead for client applications.
///
/// # Example
///
/// ```rust,ignore
/// let config = Config::load()?;
/// let infra = ServerInfra::new(&config).await?;
///
/// // Use infrastructure services
/// let user = infra.db.find_by_id(user_id).await?;
/// let customer = infra.stripe.create_customer(...).await?;
/// ```
pub struct ServerInfra {
    /// Database repository providing all data access operations
    pub db: DbRepo,
    /// GitHub API adapter for OAuth and user operations
    pub github: GitHubHttpClient,
    /// Stripe SDK for billing and payment processing (if configured)
    pub stripe: Option<StripeSdk>,
}

impl ServerInfra {
    /// Creates a new ServerInfra instance with all infrastructure services initialized
    ///
    /// # Arguments
    ///
    /// * `cfg` - Application configuration containing database URL, API keys, etc.
    ///
    /// # Errors
    ///
    /// Returns `DomainError` if:
    /// - Database connection fails
    /// - HTTP client initialization fails
    /// - Required configuration values are missing (e.g., Stripe secret key)
    pub async fn new(cfg: &common::Config) -> Result<Self, DomainError> {
        // Initialize database
        let db = DbRepo::new(&cfg.database_url)
            .await
            .map_err(|e| DomainError::Internal(format!("Database initialization failed: {e}")))?;

        // Initialize HTTP client for adapters
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(cfg.api_timeout_seconds))
            .build()
            .map_err(|e| {
                DomainError::Internal(format!("HTTP client initialization failed: {e}"))
            })?;

        // Initialize GitHub adapter
        let github = GitHubHttpClient::new(http_client.clone());

        // Initialize Stripe SDK only if configured
        // TODO: This is kind hacky, we should have a better way to handle this
        let stripe = if let Some(stripe_secret_key) = &cfg.stripe_secret_key {
            if cfg.stripe_webhook_secret.is_empty() {
                eprintln!("Warning: Stripe webhook secret is empty");
            }
            Some(StripeSdk::new(
                stripe_secret_key.clone(),
                cfg.stripe_webhook_secret.clone(),
            ))
        } else {
            None
        };

        Ok(Self { db, github, stripe })
    }
}

/// Client-safe infrastructure without sensitive credentials
///
/// This struct provides infrastructure services that are safe to use in client
/// applications (CLI, desktop apps, etc.) as they don't contain any server-side
/// secrets or credentials.
///
/// # Security Note
///
/// This struct is designed to be safe for inclusion in distributed binaries.
/// It only contains services that use user-provided credentials (like OAuth tokens)
/// rather than server-side secrets.
///
/// # Example
///
/// ```rust,ignore
/// let config = ClientConfig::load()?;
/// let infra = ClientInfra::new(&config)?;
///
/// // Use GitHub adapter with user's OAuth token
/// let user_info = infra.github.get_with_auth(url, &user_token).await?;
/// ```
pub struct ClientInfra {
    /// GitHub API adapter for OAuth operations using user-provided tokens
    pub github: GitHubHttpClient,
}

impl ClientInfra {
    /// Creates a new ClientInfra instance with client-safe infrastructure services
    ///
    /// # Arguments
    ///
    /// * `cfg` - Application configuration (only uses timeout settings, no secrets)
    ///
    /// # Errors
    ///
    /// Returns `DomainError` if HTTP client initialization fails
    pub fn new(cfg: &common::Config) -> Result<Self, DomainError> {
        // Initialize HTTP client for adapters
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(cfg.api_timeout_seconds))
            .build()
            .map_err(|e| {
                DomainError::Internal(format!("HTTP client initialization failed: {e}"))
            })?;

        // Initialize GitHub adapter (uses user's OAuth tokens, not server secrets)
        let github = GitHubHttpClient::new(http_client);

        Ok(Self { github })
    }
}
