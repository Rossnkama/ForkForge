//! # Database Infrastructure Module
//!
//! This module provides SQLite/SQLx implementations of all domain repository traits.
//! It handles database connections, migrations, and data access operations.
//!
//! ## Architecture
//!
//! - Uses SQLx for async database operations
//! - Implements all repository traits defined in the domain layer
//! - Manages database migrations via SQLx migrate macro
//! - Currently supports SQLite with plans for PostgreSQL support

use async_trait::async_trait;
use domain::errors::DomainError;
use domain::models::{AuthCredentials, ForkSession, Snapshot, User};
use domain::repositories::{AuthRepository, SessionRepository, SnapshotRepository, UserRepository};
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
pub use sqlx::sqlite::SqlitePool;
use std::str::FromStr;
use uuid::Uuid;

/// Static migrator instance for database schema management
pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// Database repository implementing all domain repository traits
///
/// This struct provides a unified interface for all database operations,
/// implementing the repository pattern to abstract data access from business logic.
#[derive(Clone)]
pub struct DbRepo {
    pool: SqlitePool,
}

impl DbRepo {
    /// Creates a new database repository with connection pool
    ///
    /// # Arguments
    ///
    /// * `database_url` - SQLite connection URL (e.g., "sqlite:./forkforge.db")
    ///
    /// # Notes
    ///
    /// - Automatically appends `?mode=rwc` if not present (read-write-create)
    /// - Creates database file if it doesn't exist
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let db_url = if database_url.starts_with("sqlite:") {
            if !database_url.contains("?mode=") {
                format!("{database_url}?mode=rwc")
            } else {
                database_url.to_string()
            }
        } else {
            return Err(sqlx::Error::Configuration(
                "Only SQLite databases are supported".into(),
            ));
        };

        let connect_options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(connect_options).await?;

        Ok(Self { pool })
    }

    /// Returns a reference to the underlying SQLite connection pool
    ///
    /// This is exposed for advanced use cases where direct pool access is needed.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Runs all pending database migrations
    ///
    /// This should be called during application startup to ensure
    /// the database schema is up to date.
    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[async_trait]
impl UserRepository for DbRepo {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, DomainError> {
        todo!("Implement find_by_id")
    }

    async fn find_by_external_id(
        &self,
        provider: &str,
        external_id: &str,
    ) -> Result<Option<User>, DomainError> {
        // In production, would query database based on provider and external_id
        let _ = (provider, external_id);
        todo!("Implement find_by_external_id")
    }

    async fn create(&self, _user: &User) -> Result<User, DomainError> {
        todo!("Implement create user")
    }

    async fn update(&self, _user: &User) -> Result<User, DomainError> {
        todo!("Implement update user")
    }
}

#[async_trait]
impl AuthRepository for DbRepo {
    async fn find_by_token_hash(
        &self,
        _token_hash: &str,
    ) -> Result<Option<AuthCredentials>, DomainError> {
        todo!("Implement find_by_token_hash")
    }

    async fn create(&self, _credentials: &AuthCredentials) -> Result<AuthCredentials, DomainError> {
        todo!("Implement create auth credentials")
    }

    async fn update_last_used(&self, _id: Uuid) -> Result<(), DomainError> {
        todo!("Implement update_last_used")
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        todo!("Implement delete auth credentials")
    }
}

#[async_trait]
impl SessionRepository for DbRepo {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ForkSession>, DomainError> {
        todo!("Implement find_by_id")
    }

    async fn find_by_user_id(&self, _user_id: Uuid) -> Result<Vec<ForkSession>, DomainError> {
        todo!("Implement find_by_user_id")
    }

    async fn create(&self, _session: &ForkSession) -> Result<ForkSession, DomainError> {
        todo!("Implement create session")
    }

    async fn update(&self, _session: &ForkSession) -> Result<ForkSession, DomainError> {
        todo!("Implement update session")
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        todo!("Implement delete session")
    }
}

#[async_trait]
impl SnapshotRepository for DbRepo {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Snapshot>, DomainError> {
        todo!("Implement find_by_id")
    }

    async fn find_by_session_id(&self, _session_id: Uuid) -> Result<Vec<Snapshot>, DomainError> {
        todo!("Implement find_by_session_id")
    }

    async fn find_by_user_id(&self, _user_id: Uuid) -> Result<Vec<Snapshot>, DomainError> {
        todo!("Implement find_by_user_id")
    }

    async fn create(&self, _snapshot: &Snapshot) -> Result<Snapshot, DomainError> {
        todo!("Implement create snapshot")
    }

    async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
        todo!("Implement delete snapshot")
    }
}

pub async fn init_db(database_url: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_repo = DbRepo::new(database_url).await?;
    db_repo.run_migrations().await?;
    Ok(db_repo.pool)
}

pub async fn list_tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    Ok(tables.into_iter().map(|(name,)| name).collect())
}

pub async fn list_migrations(pool: &SqlitePool) -> Result<Vec<(i64, String)>, sqlx::Error> {
    let migrations: Vec<(i64, String)> =
        sqlx::query_as("SELECT version, description FROM _sqlx_migrations ORDER BY version")
            .fetch_all(pool)
            .await?;

    Ok(migrations)
}
