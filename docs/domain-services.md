# Domain Services Documentation

## Overview

The domain layer contains all business logic for ForkForge, organized into focused services that handle specific business capabilities. Each service is designed to be independent, testable, and free from infrastructure concerns.

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

```rust
pub struct GitHubAuthService<C: HttpClient> {
    client_id: String,
    http_client: C,
}
```

**Key Features:**

- Device flow authentication
- Token management
- User profile retrieval
- Error handling with retry logic

**Usage Example:**

```rust
let github_service = GitHubAuthService::new(client_id, http_client);
let device_code = github_service.request_device_code().await?;
let auth_response = github_service.poll_authorization(device_code).await?;
let user = github_service.get_user(&auth_response.access_token).await?;
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
        HttpClient {}
        
        #[async_trait]
        impl HttpClient for HttpClient {
            async fn post_form(&self, url: &str, body: &str) -> Result<String, DomainError>;
            async fn get_with_auth(&self, url: &str, token: &str) -> Result<String, DomainError>;
        }
    }

    #[tokio::test]
    async fn test_github_auth() {
        let mut mock_client = MockHttpClient::new();
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

