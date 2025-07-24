# ForkForge Architecture

<!--toc:start-->
- [ForkForge Architecture](#forkforge-architecture)
  - [Overview](#overview)
  - [Architecture Diagram](#architecture-diagram)
  - [Layer Responsibilities](#layer-responsibilities)
    - [Domain Layer (`crates/domain/`)](#domain-layer-cratesdomain)
    - [Infrastructure Layer (`crates/infra/`)](#infrastructure-layer-cratesinfra)
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
│                     Uses ClientInfra                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        API Layer                            │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ HTTP Routes     │  │ Middleware   │  │ Handlers     │    │
│  └─────────────────┘  └──────────────┘  └──────────────┘    │
│                     Uses ServerInfra                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Models          │  │ Services     │  │ Repositories │    │
│  │ - User          │  │ - Auth       │  │ (Traits)     │    │
│  │ - Session       │  │ - Forking    │  │              │    │
│  │ - Snapshot      │  │ - Billing    │  │ External     │    │
│  │ - Subscription  │  │ - Snapshots  │  │ Interfaces   │    │
│  └─────────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer (infra)               │
│  ┌─────────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ Database (db)   │  │ HTTP Clients │  │ External     │    │
│  │ - DbRepo        │  │ - GitHub     │  │ Services     │    │
│  │ - Migrations    │  │   Adapter    │  │ - StripeSdk  │    │
│  └─────────────────┘  └──────────────┘  │ - Helius*    │    │
│                                         └──────────────┘    │
│  ┌────────────────────────────────────────────────────┐     │
│  │ Façade Pattern:                                    │     │
│  │ - ServerInfra (contains secrets, server-only)      │     │
│  │ - ClientInfra (no secrets, safe for CLI)           │     │
│  └────────────────────────────────────────────────────┘     │
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

### Infrastructure Layer (`crates/infra/`)

Implements all domain-defined interfaces for external services and data persistence.

**Responsibilities:**

- Database operations via `DbRepo`
- HTTP client implementations (`GitHubAdapter`)
- External service integrations (`StripeSdk`)
- Migration management

**Security Architecture:**

- **ServerInfra**: Contains all services including sensitive ones (database, Stripe secrets)
  - Used only by the API server
  - Contains credentials and API keys
- **ClientInfra**: Contains only client-safe services (GitHub adapter)
  - Safe for CLI and distributed binaries
  - No server secrets

### API Layer (`crates/api/`)

Handles HTTP communication and adapts external requests to domain operations.

**Responsibilities:**

- HTTP routing and middleware
- Request/response serialization
- Authentication and authorization
- Uses `ServerInfra` for all infrastructure needs
- Coordinates domain services

### CLI Layer (`crates/cli/`)

Provides command-line interface for users.

**Responsibilities:**

- Command parsing and validation
- User interaction and prompts
- Display formatting
- Uses `ClientInfra` for GitHub authentication
- Maintains separate HTTP client for API communication

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
└── github.rs       # Domain authentication service and traits

infra/src/
├── github_device_flow.rs  # GitHub OAuth implementation
└── github.rs             # HTTP adapter
```

**Design Principles:**

- Domain defines the `DeviceFlowProvider` trait
- Infrastructure implements provider-specific logic
- Complete separation of business rules from OAuth details
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
 │                     │                    │
 └──────depends on─────┴─────depends on─────┘
            Common     │
                       │
                    Infra
                  implements
               domain interfaces
```

**Key Points:**

- Domain has no dependencies on infrastructure
- Infrastructure implements domain-defined interfaces
- API uses ServerInfra, CLI uses ClientInfra
- All layers can use Common
- Dependency inversion: Domain defines interfaces, infra implements

## Repository Pattern

The domain defines repository traits that the infrastructure layer implements:

```rust
// Domain defines the interface (domain/src/repositories.rs)
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &User) -> Result<User, DomainError>;
}

// Infrastructure provides the implementation (infra/src/db.rs)
pub struct DbRepo {
    pool: SqlitePool,
}

impl UserRepository for DbRepo {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        // SQLx implementation details...
    }
}
```

The same pattern applies to external services:

```rust
// Domain defines what it needs (domain/src/services/auth/github.rs)
pub trait DeviceFlowProvider: Send + Sync {
    async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError>;
    async fn poll_authorization(&self, device_code: &str) -> Result<String, AuthError>;
    async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError>;
}

// Infrastructure provides the implementation (infra/src/github_device_flow.rs)
pub struct GitHubDeviceFlowProvider {
    client_id: String,
    http_client: GitHubAdapter,
}

impl DeviceFlowProvider for GitHubDeviceFlowProvider {
    // GitHub-specific OAuth implementation...
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
