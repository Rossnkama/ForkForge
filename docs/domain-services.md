# Domain Services Documentation

<!--toc:start-->
- [Domain Services Documentation](#domain-services-documentation)
  - [Overview](#overview)
  - [Service Structure](#service-structure)
    - [Directory Organization](#directory-organization)
  - [Authentication Service](#authentication-service)
    - [Purpose](#purpose)
    - [GitHub OAuth Implementation](#github-oauth-implementation)
    - [Adding New Auth Providers](#adding-new-auth-providers)
  - [Session Service](#session-service)
    - [Purpose](#purpose)
    - [Core Functions](#core-functions)
  - [Snapshot Service](#snapshot-service)
    - [Purpose](#purpose)
    - [Core Functions](#core-functions)
  - [Billing Service](#billing-service)
    - [Purpose](#purpose)
    - [Webhook Processing](#webhook-processing)
  - [Forking Service (Planned)](#forking-service-planned)
    - [Purpose](#purpose)
    - [Planned Modules](#planned-modules)
  - [Error Handling](#error-handling)
    - [Domain Error Types](#domain-error-types)
    - [Service-Specific Errors](#service-specific-errors)
  - [Repository Pattern](#repository-pattern)
    - [Purpose](#purpose)
    - [Example Repository Trait](#example-repository-trait)
    - [Benefits](#benefits)
  - [Testing Strategies](#testing-strategies)
    - [Unit Testing](#unit-testing)
    - [Integration Testing](#integration-testing)
  - [Best Practices](#best-practices)
    - [Service Design](#service-design)
    - [Code Organization](#code-organization)
    - [Future Extensibility](#future-extensibility)
  - [Implementation Status](#implementation-status)
  - [Next Steps](#next-steps)
<!--toc:end-->

## Overview

The domain layer contains all business logic for ForkForge, organized into focused services that handle specific business capabilities. Each service is designed to be independent, testable, and free from infrastructure concerns.

These domain services are used by both the API server and CLI through dependency injection, ensuring consistent business logic across all entry points to the system.

## Service Structure

### Directory Organization

```
crates/domain/src/services/
├── auth/               # Authentication providers
│   ├── mod.rs         # Public exports
│   ├── types.rs       # Shared auth types
│   └── github.rs      # GitHub OAuth implementation
├── billing/           # Payment and subscription management
│   ├── mod.rs
│   └── webhooks.rs    # Stripe webhook handling
├── forking/           # Solana validator forking (future)
│   └── mod.rs
├── snapshots/         # Snapshot management
│   └── mod.rs
└── sessions.rs        # Fork session management
```

## Authentication Service

### Purpose

Handles user authentication across multiple providers with a focus on extensibility and security.

### GitHub OAuth Implementation

**Domain Layer (`domain/src/services/auth/github.rs`):**

```rust
// Pure domain service that uses injected provider
pub struct AuthService<P: DeviceFlowProvider> {
    provider: P,
}

// Domain trait defining the contract
pub trait DeviceFlowProvider: Send + Sync {
    async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError>;
    async fn poll_authorization(&self, device_code: &str) -> Result<String, AuthError>;
    async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError>;
}
```

**Infrastructure Layer (`infra/src/github_device_flow.rs`):**

```rust
// Concrete GitHub implementation with all OAuth details
pub struct GitHubDeviceFlowProvider {
    client_id: String,
    http_client: GitHubAdapter,
}

impl DeviceFlowProvider for GitHubDeviceFlowProvider {
    // GitHub-specific URLs, polling logic, error mapping...
}
```

**Key Features:**

- Clean separation between domain and infrastructure
- Device flow authentication without domain knowing GitHub URLs
- Token management abstracted from OAuth specifics
- User profile retrieval through clean interfaces
- Infrastructure handles all polling and retry logic

**Usage Example:**

```rust
// In API server setup
let provider = GitHubDeviceFlowProvider::new(client_id, http_client);
let auth_service = AuthService::new(provider);

// Usage remains clean
let device_code = auth_service.request_device_code().await?;
let access_token = auth_service.wait_for_authorization(&device_code.device_code).await?;
let user = auth_service.get_user(&access_token).await?;
```

### Adding New Auth Providers

To add a new authentication provider (e.g., Google):

1. Create `auth/google.rs`
2. Implement provider-specific logic
3. Export from `auth/mod.rs`
4. Follow the same patterns as GitHub implementation

## Session Service

### Purpose

Manages Solana fork sessions throughout their lifecycle.

### Core Functions

```rust
pub async fn create_session(
    user_id: Uuid, 
    name: String
) -> Result<ForkSession, DomainError>
```

**Session States:**

- `Starting` - Initial state, validator being launched
- `Running` - Active fork ready for use
- `Stopped` - Gracefully shut down
- `Failed` - Error during operation

**Future Enhancements:**

- Validator process management
- Resource allocation
- Session persistence
- Automatic cleanup

## Snapshot Service

### Purpose

Enables time-travel functionality by capturing and restoring validator state.

### Core Functions

```rust
pub async fn create_snapshot(
    session_id: Uuid,
    user_id: Uuid,
    name: String,
    description: Option<String>,
) -> Result<Snapshot, DomainError>
```

**Planned Features:**

- ZFS snapshot integration
- State compression
- Snapshot sharing via URLs
- Incremental snapshots
- Snapshot marketplace

## Billing Service

### Purpose

Handles subscription management and payment processing.

### Webhook Processing

```rust
pub async fn process_stripe_webhook(
    event_type: &str,
    event_data: serde_json::Value,
) -> Result<(), DomainError>
```

**Supported Events:**

- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`

**Future Implementation:**

- Subscription tier management
- Usage tracking
- Credit system
- Invoice generation

## Forking Service (Planned)

### Purpose

Core functionality for creating Solana mainnet forks.

### Planned Modules

**`clone.rs`** - Account cloning

- Fetch accounts from mainnet RPC
- Minimal state transfer
- Account filtering

**`validator.rs`** - Validator management

- Process spawning
- Configuration generation
- Health monitoring

**`rpc.rs`** - RPC interactions

- Mainnet communication
- State queries
- Transaction forwarding

## Error Handling

### Domain Error Types

```rust
pub enum DomainError {
    NotFound(String),
    Unauthorized(String),
    InvalidInput(String),
    ExternalService(String),
    Internal(String),
}
```

### Service-Specific Errors

```rust
pub enum AuthError {
    UserAuthenticationTimeout,
    UserDeniedAuthentication,
    ServerConfigurationError { debug_info: String },
    InternalServerError { debug_info: String },
}
```

## Repository Pattern

### Purpose

Abstracts data persistence from business logic.

### Example Repository Trait

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn find_by_github_id(&self, github_id: u64) -> Result<Option<User>, DomainError>;
    async fn create(&self, user: &User) -> Result<User, DomainError>;
    async fn update(&self, user: &User) -> Result<User, DomainError>;
}
```

### Benefits

- Testability with mock implementations
- Database independence
- Clear contracts
- Easy to switch storage backends

## Testing Strategies

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        DeviceFlowProvider {}

        #[async_trait]
        impl DeviceFlowProvider for DeviceFlowProvider {
            async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError>;
            async fn poll_authorization(&self, device_code: &str) -> Result<String, AuthError>;
            async fn get_user(&self, access_token: &str) -> Result<GitHubUser, DomainError>;
        }
    }

    #[tokio::test]
    async fn test_auth_service() {
        let mut mock_provider = MockDeviceFlowProvider::new();
        mock_provider
            .expect_request_device_code()
            .returning(|| Ok(DeviceCodeResponse { /* ... */ }));
        
        let auth_service = AuthService::new(mock_provider);
        // Test implementation
    }
}
```

### Integration Testing

- Test service interactions
- Use test databases
- Mock external APIs

## Best Practices

### Service Design

1. **Single Responsibility** - Each service handles one business capability
2. **Dependency Injection** - Accept interfaces, not implementations
3. **Error Propagation** - Use Result types consistently
4. **Async by Default** - All public functions should be async

### Code Organization

1. **Public API in mod.rs** - Export only what's needed
2. **Internal modules** - Keep implementation details private
3. **Shared types** - Use types.rs for common structures
4. **Documentation** - Document public interfaces thoroughly

### Future Extensibility

1. **Trait-based design** - Easy to add new implementations
2. **Feature flags** - Enable/disable features at compile time
3. **Configuration** - Externalize all settings
4. **Monitoring** - Add metrics and logging hooks

## Implementation Status

| Service | Status | Description |
|---------|--------|-------------|
| Auth/GitHub | ✅ Implemented | Full OAuth device flow |
| Sessions | 🔨 Stub | Basic structure, needs forking logic |
| Snapshots | 🔨 Stub | Basic structure, needs storage backend |
| Billing | 🔨 Stub | Webhook structure, needs Stripe integration |
| Forking | 📋 Planned | Not yet implemented |

## Next Steps

1. **Implement Forking Service** - Core product functionality
2. **Complete Billing Integration** - Stripe webhook handling
3. **Add Storage Backends** - Implement repository traits
4. **Enhanced Error Handling** - More specific error types
5. **Performance Optimization** - Caching, connection pooling
6. **Monitoring Integration** - Metrics and tracing
