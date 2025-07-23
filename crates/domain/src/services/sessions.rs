use crate::errors::DomainError;
use crate::models::{ForkSession, SessionStatus};
use chrono::Utc;
use uuid::Uuid;

pub async fn create_session(user_id: Uuid, name: String) -> Result<ForkSession, DomainError> {
    // TODO: Implement actual session creation logic
    // This is a stub that will be expanded when forking functionality is implemented

    let session = ForkSession {
        id: Uuid::new_v4(),
        user_id,
        name,
        status: SessionStatus::Starting,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(session)
}
