use crate::errors::DomainError;
use crate::models::Snapshot;
use uuid::Uuid;

/// Domain-defined contract for snapshot management
#[async_trait::async_trait]
pub trait SnapshotRepository: Send + Sync {
    /// Create a new snapshot
    async fn create(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Snapshot, DomainError>;

    /// Find snapshot by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Snapshot>, DomainError>;

    /// Find snapshots by user ID
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;

    /// Find snapshots by session ID
    async fn find_by_session(&self, session_id: Uuid) -> Result<Vec<Snapshot>, DomainError>;

    /// Delete snapshot
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

/// Domain service for snapshot operations
pub struct SnapshotService<R: SnapshotRepository> {
    repository: R,
}

impl<R: SnapshotRepository> SnapshotService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Create a new snapshot
    pub async fn create_snapshot(
        &self,
        session_id: Uuid,
        user_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Snapshot, DomainError> {
        self.repository
            .create(session_id, user_id, name, description)
            .await
    }

    /// Get snapshot by ID
    pub async fn get_snapshot(&self, id: Uuid) -> Result<Option<Snapshot>, DomainError> {
        self.repository.find_by_id(id).await
    }

    /// List user's snapshots
    pub async fn list_user_snapshots(&self, user_id: Uuid) -> Result<Vec<Snapshot>, DomainError> {
        self.repository.find_by_user(user_id).await
    }

    /// List session's snapshots
    pub async fn list_session_snapshots(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Snapshot>, DomainError> {
        self.repository.find_by_session(session_id).await
    }

    /// Delete snapshot
    pub async fn delete_snapshot(&self, id: Uuid) -> Result<(), DomainError> {
        self.repository.delete(id).await
    }
}
