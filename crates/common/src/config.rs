use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // API
    #[serde(default = "default_api_host")]
    pub api_host: String,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub stripe_webhook_secret: String,
    #[serde(default = "default_api_timeout_seconds")]
    pub api_timeout_seconds: u64,

    // Stripe
    pub stripe_publishable_key: Option<String>,
    pub stripe_secret_key: Option<String>,
    pub stripe_product_id_entry_tier: Option<String>,
    pub stripe_product_id_lite_tier: Option<String>,
    pub stripe_product_id_pro_tier: Option<String>,

    // Github
    pub github_client_id: Option<String>,
    pub github_client_secret: Option<String>,
}

fn default_api_host() -> String {
    "127.0.0.1".to_string()
}

fn default_api_port() -> u16 {
    3000
}

fn default_api_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_database_url() -> String {
    "sqlite://forkforge.db".to_string()
}

fn default_api_timeout_seconds() -> u64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_host: default_api_host(),
            api_port: default_api_port(),
            api_base_url: default_api_base_url(),
            database_url: default_database_url(),
            stripe_webhook_secret: String::new(),
            api_timeout_seconds: default_api_timeout_seconds(),
            stripe_publishable_key: None,
            stripe_secret_key: None,
            stripe_product_id_entry_tier: None,
            stripe_product_id_lite_tier: None,
            stripe_product_id_pro_tier: None,
            github_client_id: None,
            github_client_secret: None,
        }
    }
}

impl Config {
    pub fn figment() -> Figment {
        Figment::new()
            // Start with default values
            .merge(Serialized::defaults(Config::default()))
            // Load from config.toml (profile-aware)
            .merge(Toml::file("config.toml").nested())
            // Environment variables override everything
            .merge(Env::prefixed("FORKFORGE_"))
    }

    pub fn from_profile(profile: &str) -> Result<Self, Box<figment::Error>> {
        Ok(Self::figment().select(profile).extract()?)
    }

    pub fn load() -> Result<Self, Box<figment::Error>> {
        // Try to get profile from env var, default to "default"
        let profile = std::env::var("FORKFORGE_PROFILE").unwrap_or_else(|_| "default".to_string());
        Self::from_profile(&profile)
    }
}
