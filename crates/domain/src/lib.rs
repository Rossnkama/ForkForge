//! # Domain Layer
//!
//! This crate contains the core business logic for ForkForge/Chainbox, following
//! Domain-Driven Design principles. It is completely independent of infrastructure
//! concerns and defines interfaces (traits) that are implemented by the infra layer.
//!
//! ## Architecture Principles
//!
//! - **Pure Business Logic**: No I/O operations, database queries, or external service calls
//! - **Dependency Inversion**: Defines interfaces (traits) that infrastructure implements
//! - **Framework Agnostic**: No dependencies on web frameworks, ORMs, or external libraries
//! - **Testable**: All business logic can be unit tested with mock implementations
//!
//! ## Module Structure
//!
//! - `errors`: Domain-specific error types
//! - `models`: Core domain entities (User, Session, Snapshot, etc.)
//! - `repositories`: Data access interfaces (traits)
//! - `services`: Business logic and use cases

pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;
