use serde::{Deserialize, Serialize};

/// Minimal configuration for the CLI client - contains NO secrets
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,

    #[serde(default = "default_api_timeout_seconds")]
    pub api_timeout_seconds: u64,

    #[serde(skip)]
    pub http_client: reqwest::Client,

    #[serde(skip)]
    pub long_poll_client: reqwest::Client,
}

fn default_api_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_api_timeout_seconds() -> u64 {
    30
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_base_url: default_api_base_url(),
            api_timeout_seconds: default_api_timeout_seconds(),
            http_client: reqwest::Client::new(),
            long_poll_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(900))
                .build()
                .expect("Failed to build long poll client"),
        }
    }
}

impl ClientConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Start with defaults
        let mut config = Self::default();

        // Only check environment variables - no config file access
        if let Ok(url) = std::env::var("FORKFORGE_API_BASE_URL") {
            config.api_base_url = url;
        }

        if let Ok(timeout) = std::env::var("FORKFORGE_API_TIMEOUT_SECONDS") {
            if let Ok(seconds) = timeout.parse::<u64>() {
                config.api_timeout_seconds = seconds;
                // Rebuild clients with new timeout
                config.http_client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(seconds))
                    .build()?;
            }
        }

        Ok(config)
    }
}
