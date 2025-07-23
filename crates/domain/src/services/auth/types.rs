use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeRequest {
    pub client_id: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckAuthorisationRequest {
    pub client_id: String,
    pub device_code: String,
    pub grant_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckAuthorisationResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub enum AuthError {
    UserAuthenticationTimeout,
    UserDeniedAuthentication,
    ServerConfigurationError { debug_info: String },
    InternalServerError { debug_info: String },
}

impl AuthError {
    pub fn message(&self) -> String {
        match self {
            AuthError::UserAuthenticationTimeout => {
                "Authentication timed out. Please try logging in again.".to_string()
            }
            AuthError::UserDeniedAuthentication => {
                "Authentication was denied. Please check your permissions and try again."
                    .to_string()
            }
            AuthError::ServerConfigurationError { debug_info } => {
                #[cfg(debug_assertions)]
                {
                    format!("Server configuration error. [DEBUG: {}]", debug_info)
                }
                #[cfg(not(debug_assertions))]
                {
                    let _ = debug_info;
                    "Something went wrong on our end. We're looking into it.".to_string()
                }
            }
            AuthError::InternalServerError { debug_info } => {
                #[cfg(debug_assertions)]
                {
                    format!("Internal server error. [DEBUG: {}]", debug_info)
                }
                #[cfg(not(debug_assertions))]
                {
                    let _ = debug_info;
                    "Something went wrong on our end. We're looking into it.".to_string()
                }
            }
        }
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for AuthError {}
