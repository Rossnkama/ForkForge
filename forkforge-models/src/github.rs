use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeRequestParams {
    /// OAuth app client ID from GitHub
    pub client_id: String,
    /// Space-delimited list of scopes (e.g., "repo user")
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    /// Code used to poll for access token
    pub device_code: String,
    /// Seconds until device_code expires (typically 900)
    #[serde(rename = "expires_in")]
    pub _expires_in: u32,
    /// Minimum seconds to wait between polling requests
    #[serde(rename = "interval")]
    pub _interval: u32,
    /// Short code shown to user (e.g., "ABCD-1234")
    pub user_code: String,
    /// URL where user enters the user_code (typically https://github.com/login/device)
    pub verification_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckUserAuthorisedRequestParams {
    /// OAuth app client ID from GitHub
    pub client_id: String,
    /// Device code from the initial authorization request
    pub device_code: String,
    /// Must be "urn:ietf:params:oauth:grant-type:device_code"
    pub grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckUserAuthorisedResponse {
    /// GitHub personal access token for authenticated API requests
    pub access_token: String,
    /// Token type (typically "bearer")
    #[serde(rename = "token_type")]
    pub _token_type: String,
    /// Granted scopes (may differ from requested)
    #[serde(rename = "scope")]
    pub _scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    /// Unique GitHub user ID (numeric)
    pub id: u64,
    /// The GitHub username of the repository owner
    pub login: String,
}
