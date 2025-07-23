# ForkForge Development Guide

## Getting Started

### Prerequisites

Before you begin development on ForkForge, ensure you have the following installed:

- **Rust** 1.75+ (2024 edition)
- **Cargo** (comes with Rust)
- **SQLite3** (for local development)
- **Git**

### Initial Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/forkforge.git
   cd forkforge
   ```

2. **Set up configuration**

   ```bash
   # Copy the example configuration
   cp config.toml.example config.toml
   
   # Edit config.toml with your settings
   # You'll need:
   # - GitHub OAuth App credentials
   # - Stripe API keys (for billing features)
   ```

3. **Initialize the database**

   ```bash
   # Simple migration runner
   cargo run --bin migrate
   
   # Or use the detailed initialization tool
   cargo run --bin db-init
   ```

4. **Verify your setup**

   ```bash
   # Run all tests
   cargo test
   
   # Check code compilation
   cargo check
   ```

## Project Structure

### Workspace Organization

```
forkforge/
├── Cargo.toml              # Workspace manifest
├── config.toml             # Application configuration
├── migrations/             # SQL migration files
├── docs/                   # Documentation
└── crates/                 # Rust crates
    ├── domain/            # Business logic
    ├── api/               # HTTP API server
    ├── cli/               # Command-line interface
    └── common/            # Shared components
```

### Understanding the Crates

#### Domain Crate

- **Purpose**: Core business logic, independent of infrastructure
- **Key concepts**: Services, Models, Repository traits
- **No dependencies on**: HTTP, Database drivers, UI frameworks

#### API Crate

- **Purpose**: HTTP server implementation
- **Framework**: Axum
- **Responsibilities**: Routes, middleware, database implementation

#### CLI Crate

- **Purpose**: Command-line interface
- **Framework**: Clap
- **Features**: User prompts, API client, display formatting

#### Common Crate

- **Purpose**: Shared utilities and DTOs
- **Contents**: Configuration, data transfer objects

## Development Workflow

### Running the Application

```bash
# Start the API server
cargo run --bin api

# In another terminal, run CLI commands
cargo run --bin cli -- login
cargo run --bin cli -- up
```

### Development Mode with Auto-Reload

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and auto-restart API server
cargo watch -x "run --bin api"

# Watch and run tests
cargo watch -x test
```

### Code Quality

Before committing code, ensure it passes all quality checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Check for security issues
cargo audit
```

## Adding New Features

### 1. Adding a New Authentication Provider

Example: Adding Google OAuth

```rust
// 1. Create domain/src/services/auth/google.rs
pub struct GoogleAuthService<C: HttpClient> {
    client_id: String,
    http_client: C,
}

impl<C: HttpClient> GoogleAuthService<C> {
    pub async fn authenticate(&self) -> Result<GoogleUser, AuthError> {
        // Implementation
    }
}

// 2. Export from domain/src/services/auth/mod.rs
pub mod google;

// 3. Add route in api/src/server.rs
.route("/auth/google/login", post(google_login))
```

### 2. Adding a New Domain Service

```rust
// 1. Create service module
// domain/src/services/analytics/mod.rs
pub async fn track_usage(
    user_id: Uuid,
    action: String,
) -> Result<(), DomainError> {
    // Implementation
}

// 2. Define repository trait if needed
// domain/src/repositories.rs
#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn record_event(&self, event: &Event) -> Result<(), DomainError>;
}

// 3. Implement in API layer
// api/src/repositories/analytics.rs
```

### 3. Adding a New CLI Command

```rust
// 1. Add to cli/src/client.rs
#[derive(Subcommand)]
enum Commands {
    Login,
    Up,
    Status,  // New command
}

// 2. Implement handler
async fn handle_status(config: ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation
}

// 3. Add to match statement in main()
Some(Commands::Status) => {
    handle_status(config).await?;
}
```

## Database Development

### Creating a New Migration

```bash
# Add timestamp-prefixed migration file
echo "CREATE TABLE analytics (...);" > migrations/$(date +%Y%m%d)_000001_add_analytics.sql

# Run migrations
cargo run --bin migrate
```

### Database Schema Guidelines

1. **Use UUIDs for primary keys** (stored as TEXT in SQLite)
2. **Add indexes for foreign keys and commonly queried fields**
3. **Use CHECK constraints for enums**
4. **Always include created_at timestamps**

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let session = create_session(
            Uuid::new_v4(),
            "test-session".to_string()
        ).await.unwrap();
        
        assert_eq!(session.status, SessionStatus::Starting);
    }
}
```

### Integration Testing

```rust
// tests/api_integration.rs
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    let response = app.get("/health").await;
    assert_eq!(response.status(), 200);
}
```

### Mocking External Services

```rust
use mockall::mock;

mock! {
    StripeClient {}
    
    impl StripeClient for StripeClient {
        async fn create_subscription(&self, user_id: &str) -> Result<Subscription, Error>;
    }
}
```

## Configuration Management

### Environment-Specific Settings

```toml
# config.toml
[default]
log_level = "debug"
api_timeout_seconds = 30

[test]
database_url = "sqlite::memory:"
log_level = "warn"

[prod]
database_url = "postgres://..."
log_level = "info"
```

### Using Configuration in Code

```rust
use common::Config;

let config = Config::load()?;
println!("API running on port: {}", config.api_port);
```

## Error Handling

### Domain Errors

```rust
use domain::errors::DomainError;

pub async fn process_payment(amount: u64) -> Result<Payment, DomainError> {
    if amount == 0 {
        return Err(DomainError::InvalidInput("Amount must be positive".into()));
    }
    // Process payment
}
```

### API Error Responses

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0 {
            DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            DomainError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };
        
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

## Performance Considerations

### Async Best Practices

```rust
// Good: Concurrent requests
let (user, subscription) = tokio::join!(
    repo.find_user(id),
    repo.find_subscription(id)
);

// Avoid: Sequential awaits when unnecessary
let user = repo.find_user(id).await?;
let subscription = repo.find_subscription(id).await?;
```

### Database Optimization

```rust
// Use prepared statements
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = ?",
    user_id
)
.fetch_optional(&pool)
.await?;
```

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --bin api
```

### Use the `dbg!` macro

```rust
let result = some_function();
dbg!(&result);  // Prints file, line, and value
```

### SQL Query Logging

```bash
# Enable SQLx query logging
RUST_LOG=sqlx=debug cargo run
```

## Common Issues and Solutions

### Issue: "GitHub client ID not configured"

**Solution**: Ensure your `config.toml` has the GitHub OAuth credentials set.

### Issue: Database connection errors

**Solution**: Check that your database file exists and has correct permissions.

### Issue: Compilation errors after updating dependencies

**Solution**: Run `cargo clean && cargo build` to rebuild from scratch.

## Contributing Guidelines

1. **Create feature branches** from `main`
2. **Follow Rust conventions** - use `cargo fmt` and `cargo clippy`
3. **Write tests** for new functionality
4. **Update documentation** when adding features
5. **Keep commits focused** - one logical change per commit
6. **Use conventional commits** - e.g., "feat: add snapshot restore"

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

