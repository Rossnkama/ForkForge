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
use crate::models::{AuthCredentials, ForkSession, Snapshot, User};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository for user data operations
///
/// Handles all user-related database operations including creation,
/// retrieval by various identifiers, and updates.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;

    /// Find user by external provider ID
    ///
    /// Examples:
    /// - find_by_external_id("github", "12345")
    /// - find_by_external_id("stripe", "cus_abc123")
    async fn find_by_external_id(
        &self,
        provider: &str,
        external_id: &str,
    ) -> Result<Option<User>, DomainError>;

    async fn create(&self, user: &User) -> Result<User, DomainError>;
    async fn update(&self, user: &User) -> Result<User, DomainError>;
}

/// Repository for authentication credentials
///
/// Manages API tokens and authentication credentials with secure
/// token hashing and usage tracking.
#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AuthCredentials>, DomainError>;
    async fn create(&self, credentials: &AuthCredentials) -> Result<AuthCredentials, DomainError>;
    async fn update_last_used(&self, id: Uuid) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

/// Repository for fork session management
///
/// Handles Solana validator fork sessions including creation,
/// retrieval, and lifecycle management.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ForkSession>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<ForkSession>, DomainError>;
    async fn create(&self, session: &ForkSession) -> Result<ForkSession, DomainError>;
    async fn update(&self, session: &ForkSession) -> Result<ForkSession, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

/// Repository for snapshot management
///
/// Manages time-travel snapshots of Solana validator states,
/// enabling users to save and restore specific blockchain states.
#[async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Snapshot>, DomainError>;
    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;
    async fn create(&self, snapshot: &Snapshot) -> Result<Snapshot, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
