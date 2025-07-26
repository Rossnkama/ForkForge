//! # Repository Traits
//!
//! This module defines the data access interfaces that must be implemented
//! by the infrastructure layer. Following the Repository pattern, these traits
//! abstract all data persistence operations.
//!
//! ## Design Principles
//!
//! - All traits are async and use `async_trait` for compatibility
//! - Methods return `Result<T, DomainError>` for consistent error handling
//! - Traits require `Send + Sync` for use in async contexts
//! - No implementation details or database-specific types

use crate::errors::DomainError;
use crate::models::{AuthToken, User};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository for user data operations
///
/// Handles all user-related database operations including creation,
/// retrieval by various identifiers, and updates.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_github_id(&self, github_id: i64) -> Result<Option<User>, DomainError>;
    async fn find_by_stripe_customer_id(
        &self,
        stripe_customer_id: &str,
    ) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &User) -> Result<User, DomainError>;
    async fn update(&self, user: &User) -> Result<User, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

/// Repository for authentication tokens
///
/// Manages API tokens with secure token hashing and usage tracking.
#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<AuthToken>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<AuthToken>, DomainError>;
    async fn create(&self, token: &AuthToken) -> Result<AuthToken, DomainError>;
    async fn update_last_used(&self, id: Uuid) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn delete_expired(&self) -> Result<u64, DomainError>;
}

/// Repository for Github data
#[async_trait]
pub trait GithubRepository: Send + Sync {
    async fn find_by_user_id(&self, id: i64) -> Result<Option<User>, DomainError>;
}
