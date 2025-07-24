# ForkForge Architecture

<!--toc:start-->
- [ForkForge Architecture](#forkforge-architecture)
  - [Overview](#overview)
  - [Architecture Diagram](#architecture-diagram)
  - [Layer Responsibilities](#layer-responsibilities)
    - [Domain Layer (`crates/domain/`)](#domain-layer-cratesdomain)
    - [API Layer (`crates/api/`)](#api-layer-cratesapi)
    - [CLI Layer (`crates/cli/`)](#cli-layer-cratescli)
    - [Common Layer (`crates/common/`)](#common-layer-cratescommon)
  - [Service Architecture](#service-architecture)
    - [Authentication Service](#authentication-service)
    - [Complex Services Pattern](#complex-services-pattern)
  - [Dependency Flow](#dependency-flow)
  - [Repository Pattern](#repository-pattern)
  - [Error Handling](#error-handling)
    - [Domain Errors](#domain-errors)
    - [API Error Mapping](#api-error-mapping)
  - [Configuration Management](#configuration-management)
  - [Testing Strategy](#testing-strategy)
    - [Unit Tests](#unit-tests)
    - [Integration Tests](#integration-tests)
    - [End-to-End Tests](#end-to-end-tests)
  - [Future Considerations](#future-considerations)
    - [Scalability](#scalability)
    - [Extensibility](#extensibility)
    - [Maintainability](#maintainability)
<!--toc:end-->

## Overview

ForkForge follows Clean Architecture principles to ensure maintainability, testability, and clear separation of concerns. The architecture is designed to be flexible and extensible while keeping the core business logic independent of external frameworks and infrastructure.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Command Parser  │  │ API Client   │  │ UI/Display   │    │
│  └─────────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        API Layer                            │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ HTTP Routes     │  │ Middleware   │  │ Adapters     │    │
│  └─────────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Models          │  │ Services     │  │ Repositories │    │
│  │ - User          │  │ - Auth       │  │ (Traits)     │    │
│  │ - Session       │  │ - Forking    │  └──────────────┘    │
│  │ - Snapshot      │  │ - Billing    │                      │
│  │ - Subscription  │  │ - Snapshots  │                      │
│  └─────────────────┘  └──────────────┘                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure                           │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Database        │  │ External APIs│  │ File System  │    │
│  │ (SQLite/PG)     │  │ (GitHub,     │  │ (Snapshots)  │    │
│  └─────────────────┘  │  Stripe)     │  └──────────────┘    │
│                       └──────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

## Layer Responsibilities

### Domain Layer (`crates/domain/`)

The heart of the application containing all business logic and rules.

**Characteristics:**

- No dependencies on external frameworks
- Pure business logic
- Defines interfaces (traits) for external services
- Contains all domain models and business rules

**Components:**

- **Models**: Entity definitions (User, Session, Snapshot, etc.)
- **Services**: Business logic implementation
- **Repository Traits**: Interfaces for data persistence
- **Errors**: Domain-specific error types

### API Layer (`crates/api/`)

Handles HTTP communication and adapts external requests to domain operations.

**Responsibilities:**

- HTTP routing and middleware
- Request/response serialization
- Authentication and authorization
- Adapting domain services for HTTP
- Implementing repository traits

### CLI Layer (`crates/cli/`)

Provides command-line interface for users.

**Responsibilities:**

- Command parsing and validation
- User interaction and prompts
- Display formatting
- Uses domain services directly via dependency injection
- Infrastructure adapters for domain contracts (e.g., HTTP client)

### Common Layer (`crates/common/`)

Shared components used across layers.

**Contents:**

- Data Transfer Objects (DTOs)
- Configuration management
- Shared utilities

## Service Architecture

### Authentication Service

```
domain/services/auth/
├── mod.rs          # Public interface
├── types.rs        # Shared auth types
└── github.rs       # GitHub OAuth implementation
```

**Design Principles:**

- Each auth provider gets its own module
- Shared types in `types.rs`
- Easy to add new providers (Google, Twitter, etc.)

### Complex Services Pattern

For services with multiple components:

```
domain/services/forking/
├── mod.rs          # Public API and orchestration
├── clone.rs        # Account cloning logic
├── validator.rs    # Validator management
└── rpc.rs          # RPC client implementation
```

## Dependency Flow

```
CLI ──depends on──> Domain <──depends on── API
 │                     ▲                    │
 └──────depends on─────┴─────depends on────┘
                    Common
```

**Key Points:**

- Domain has no dependencies on infrastructure
- API and CLI depend on Domain
- All layers can use Common
- Dependency inversion: Domain defines interfaces, others implement

## Repository Pattern

The domain defines repository traits that the API layer implements:

```rust
// Domain defines the interface
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &User) -> Result<User, DomainError>;
}

// API provides the implementation
struct SqlxUserRepository {
    pool: SqlitePool,
}

impl UserRepository for SqlxUserRepository {
    // Implementation details...
}
```

## Error Handling

### Domain Errors

```rust
pub enum DomainError {
    NotFound(String),
    Unauthorized(String),
    InvalidInput(String),
    ExternalService(String),
    Internal(String),
}
```

### API Error Mapping

The API layer maps domain errors to HTTP status codes:

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.0 {
            AuthError::UserAuthenticationTimeout => StatusCode::REQUEST_TIMEOUT,
            AuthError::UserDeniedAuthentication => StatusCode::UNAUTHORIZED,
            // ...
        }
    }
}
```

## Configuration Management

Using Figment for hierarchical configuration:

1. Default values in code
2. Profile-specific values from `config.toml`
3. Environment variable overrides

```toml
[default]
api_port = 3000

[prod]
api_port = 8080
```

## Testing Strategy

### Unit Tests

- Domain logic tested in isolation
- Mock repository implementations
- No external dependencies

### Integration Tests

- Test API endpoints
- Use test database
- Mock external services

### End-to-End Tests

- Test CLI commands
- Full system interaction
- Real database and services

## Future Considerations

### Scalability

- Domain services are stateless
- Easy to add new services
- Repository pattern allows switching databases

### Extensibility

- New auth providers: Add module to `auth/`
- New features: Add service to `services/`
- New storage: Implement repository traits

### Maintainability

- Clear boundaries between layers
- Business logic isolated from infrastructure
- Easy to understand and modify
