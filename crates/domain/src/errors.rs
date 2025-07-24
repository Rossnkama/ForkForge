use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    NotFound(String),
    Unauthorized(String),
    InvalidInput(String),
    ExternalService(String),
    Internal(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "Not found: {msg}"),
            DomainError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            DomainError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            DomainError::ExternalService(msg) => write!(f, "External service error: {msg}"),
            DomainError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        DomainError::Internal(err.to_string())
    }
}
