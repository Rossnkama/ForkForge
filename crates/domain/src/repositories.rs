use crate::errors::DomainError;
use crate::models::{AuthCredentials, ForkSession, Snapshot, User};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_github_id(&self, github_id: u64) -> Result<Option<User>, DomainError>;
    async fn find_by_stripe_id(&self, stripe_id: &str) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &User) -> Result<User, DomainError>;
    async fn update(&self, user: &User) -> Result<User, DomainError>;
}

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

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ForkSession>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<ForkSession>, DomainError>;
    async fn create(&self, session: &ForkSession) -> Result<ForkSession, DomainError>;
    async fn update(&self, session: &ForkSession) -> Result<ForkSession, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Snapshot>, DomainError>;
    async fn find_by_session_id(&self, session_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;
    async fn create(&self, snapshot: &Snapshot) -> Result<Snapshot, DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
