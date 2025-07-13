use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_api_host")]
    pub api_host: String,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default)]
    pub stripe_webhook_secret: String,
    #[serde(default = "default_api_timeout_seconds")]
    pub api_timeout_seconds: u64,
}

fn default_api_host() -> String {
    "127.0.0.1".to_string()
}

fn default_api_port() -> u16 {
    3000
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
            database_url: default_database_url(),
            stripe_webhook_secret: String::new(),
            api_timeout_seconds: default_api_timeout_seconds(),
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
