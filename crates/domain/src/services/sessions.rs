use crate::errors::DomainError;
use crate::models::ForkSession;
use uuid::Uuid;

/// Domain-defined contract for session management
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create a new fork session
    async fn create(&self, user_id: Uuid, name: String) -> Result<ForkSession, DomainError>;

    /// Find session by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ForkSession>, DomainError>;

    /// Update session
    async fn update(&self, session: &ForkSession) -> Result<ForkSession, DomainError>;
}

/// Domain service for session operations
pub struct SessionService<R: SessionRepository> {
    repository: R,
}

impl<R: SessionRepository> SessionService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Create a new fork session
    pub async fn create_session(
        &self,
        user_id: Uuid,
        name: String,
    ) -> Result<ForkSession, DomainError> {
        self.repository.create(user_id, name).await
    }

    /// Get session by ID
    pub async fn get_session(&self, id: Uuid) -> Result<Option<ForkSession>, DomainError> {
        self.repository.find_by_id(id).await
    }

    /// Update existing session
    pub async fn update_session(&self, session: &ForkSession) -> Result<ForkSession, DomainError> {
        self.repository.update(session).await
    }
}
