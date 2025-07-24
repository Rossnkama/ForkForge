# ForkForge (Chainbox)

<!--toc:start-->
- [ForkForge (Chainbox)](#forkforge-chainbox)
  - [Overview](#overview)
  - [Project Structure](#project-structure)
  - [Architecture](#architecture)
    - [Clean Architecture Design](#clean-architecture-design)
    - [Key Features](#key-features)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Initial Setup](#initial-setup)
    - [Running the API Server](#running-the-api-server)
    - [Running the CLI](#running-the-cli)
  - [Configuration](#configuration)
    - [Configuration File](#configuration-file)
    - [Environment Variables](#environment-variables)
  - [Development](#development)
    - [Building](#building)
    - [Testing](#testing)
    - [Code Quality](#code-quality)
    - [Database Migrations](#database-migrations)
  - [Domain Services](#domain-services)
    - [Authentication Service](#authentication-service)
    - [Forking Service (Coming Soon)](#forking-service-coming-soon)
    - [Billing Service](#billing-service)
    - [Snapshot Service](#snapshot-service)
  - [Contributing](#contributing)
    - [Development Guidelines](#development-guidelines)
  - [Roadmap](#roadmap)
  - [License](#license)
  - [Acknowledgments](#acknowledgments)
<!--toc:end-->

Fast Solana mainnet forking CLI for local development and testing.

## Overview

ForkForge is an open-source Rust CLI that creates state-accurate Solana mainnet forks in <10s with account cloning, time-travel snapshots, and integrated debugging. Clone only required accounts from mainnet, not the entire 100TB+ state.

## Project Structure

```
forkforge/
├── crates/
│   ├── api/         # Axum-based API server
│   ├── cli/         # Command-line interface tool
│   ├── common/      # Shared DTOs (Data Transfer Objects) and configuration
│   ├── domain/      # Core business logic and domain models
│   └── infra/       # Infrastructure implementations (DB, HTTP, external services)
├── migrations/      # Database migrations
├── docs/           # Project documentation
├── config.toml     # Configuration file
└── Cargo.toml      # Workspace configuration
```

## Architecture

### Clean Architecture Design

The project follows clean architecture principles with clear separation of concerns:

- **Domain Layer** (`crates/domain/`): Core business logic, free from I/O concerns
  - Models: User, Session, Snapshot, Subscription entities
  - Services: Authentication, Forking, Billing, Snapshots
  - Repository traits for data access abstraction
  - Defines interfaces for external services (e.g., StripeClient, DeviceFlowProvider)

```rust
// Clean: Domain defines the contract, no implementation details
pub trait DeviceFlowProvider: Send + Sync {
    async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError>;
    async fn poll_authorization(&self, device_code: &str) -> Result<String, AuthError>;
}

// Without this layer: Business logic mixed with HTTP calls
pub async fn authenticate_user(client: &reqwest::Client) {
    // ❌ Domain logic coupled to HTTP library
    let response = client.post("https://github.com/login/device/code")
        .form(&[("client_id", "abc123")])
        .send().await?;
    // ❌ GitHub URLs hardcoded in business logic
}
```

- **Infrastructure Layer** (`crates/infra/`): Implementation of domain interfaces
  - Database repository implementations (SQLx-based)
  - `GitHubHttpClient` for HTTP operations
  - `GitHubDeviceFlowProvider` for OAuth device flow
  - External service integrations (`StripeSdk`)
  - Single `MIGRATOR` source for all database migrations
  - Provides `ServerInfra` façade for server-side services
  - Provides `ClientInfra` façade for client-safe services

```rust
// Clean: Infrastructure implements domain traits (actual from crates/infra/src/github_device_flow.rs)
impl DeviceFlowProvider for GitHubDeviceFlowProvider {
    async fn request_device_code(&self) -> Result<DeviceCodeResponse, DomainError> {
        let request = DeviceCodeRequest {
            client_id: self.client_id.clone(),
            scope: "user".to_owned(),
        };

        let body = serde_urlencoded::to_string(&request)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize: {e}")))?;

        // All GitHub-specific details (URLs, headers) handled here
        let response_text = self.http_client
            .post_form(GITHUB_DEVICE_CODE_REQUEST_URL, &body)
            .await?;

        serde_json::from_str(&response_text)
            .map_err(|e| DomainError::ExternalService(format!("Failed to parse: {e}")))
    }
}

// Without this layer: GitHub details spread everywhere
pub async fn get_device_code(client_id: String) {
    // ❌ Hardcoded URLs in business logic
    let url = "https://github.com/login/device/code";
    // ❌ HTTP details mixed with domain logic
    // ❌ No reusable HTTP client with connection pooling
}
```

- **API Layer** (`crates/api/`): HTTP server implementation
  - Axum-based REST API
  - Uses `ServerInfra` for all infrastructure needs
  - Handles HTTP routing and request/response transformation

```rust
// Clean: API layer only handles HTTP concerns (actual code from crates/api/src/github.rs)
pub async fn github_create_user_device_session(
    State(state): State<AppState>,
) -> Result<Json<DeviceCodeResponse>, StatusCode> {
    let domain_response = state
        .github_auth_service
        .request_device_code()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(domain_response))
}

// Without this layer: Business logic mixed with HTTP
async fn github_login() {
    // ❌ Direct GitHub API calls in handler
    let response = reqwest::Client::new()
        .post("https://github.com/login/device/code")
        .form(&[("client_id", "abc123")])
        .send().await?;
    // ❌ Parsing and error handling mixed with HTTP routing
}
```

- **CLI Layer** (`crates/cli/`): Command-line interface
  - User interaction and display logic
  - Uses `ClientInfra` for GitHub authentication
  - Maintains its own HTTP client for API communication

```rust
// Clean: CLI uses safe client infrastructure (actual code from crates/cli/src/client.rs)
async fn handle_login(config: ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Get device code via API (no direct GitHub access)
    let device_auth_data = get_device_code(&config).await?;

    // Step 2: Show user where to authenticate
    github::prompt_user_to_verify(&device_auth_data).await;

    // Step 3: Poll API server (which handles the OAuth flow)
    let auth_response = poll_for_authorization(&config, device_auth_data.device_code).await?;

    // Step 4: Get user info (actual code continues...)
    let user: GitHubUser = github::get_user_info(&auth_response.access_token, &api_service).await?;
    println!("Logging in to user {}... who has ID {}", user.login, user.id);
}

// Without this layer: CLI directly accesses sensitive resources
async fn handle_login() {
    // ❌ Direct database access in CLI
    let pool = SqlitePool::connect("sqlite:./prod.db").await?;
    // ❌ Stripe secrets in client binary
    let stripe_key = std::env::var("STRIPE_SECRET_KEY")?;
}
```

- **Common Layer** (`crates/common/`): Shared components
  - Data Transfer Objects (DTOs)
  - Configuration management with Figment

```rust
// Clean: Shared DTOs used across all layers (actual from crates/common/src/github.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    /// Code used to poll for access token
    pub device_code: String,
    /// Short code shown to user (e.g., "ABCD-1234")
    pub user_code: String,
    /// URL where user enters the user_code
    pub verification_uri: String,
    #[serde(rename = "expires_in")]
    pub _expires_in: u32,
    #[serde(rename = "interval")]
    pub _interval: u32,
}

// Without this layer: Duplicate type definitions
// ❌ API defines: struct ApiDeviceCode { device_code: String, ... }
// ❌ CLI defines: struct CliDeviceResponse { code: String, ... }
// ❌ Domain defines: struct DeviceCodeEntity { id: Uuid, code: String, ... }
// ❌ Endless conversion code between incompatible types
```

### Key Features

- **Domain-Driven Design**: Business logic isolated in domain crate
- **Dependency Inversion**: Domain defines interfaces, infrastructure implements them
- **Security-First Design**: Separate `ServerInfra` and `ClientInfra` to prevent secret leakage
- **Profile-based Configuration**: Environment-specific settings via `config.toml`
- **GitHub OAuth Integration**: Device flow authentication
- **Extensible Service Architecture**: Easy to add new auth providers and services
- **Infrastructure Abstraction**: All I/O operations centralized in infra crate

## Getting Started

### Prerequisites

- Rust 1.75+ (2024 edition)
- Cargo
- SQLite (for development)

### Initial Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/forkforge.git
   cd forkforge
   ```

2. Set up configuration:

   ```bash
   cp config.toml.example config.toml
   # Edit config.toml with your settings
   ```

3. Initialize the database:

   ```bash
   cargo run --bin db-init
   ```

### Running the API Server

```bash
cargo run --bin api
```

The API server starts on `http://127.0.0.1:3000` by default.

Available endpoints:

- `POST /auth/github/device-code` - Initiate GitHub device flow
- `POST /auth/github/wait-for-authorization` - Poll for authorization
- `GET /auth/github-login` - Get user info with access token
- `GET /health` - Health check
- `POST /sessions` - Create new fork session
- `POST /snapshots/:id` - Create snapshot
- `POST /billing/webhook` - Stripe webhook

### Running the CLI

```bash
# Login with GitHub
cargo run --bin cli -- login

# Launch a forked validator (coming soon)
cargo run --bin cli -- up
```

## Configuration

### Configuration File

Edit `config.toml` to configure the application:

```toml
[default]
# API Configuration
api_host = "127.0.0.1"
api_port = 3000
api_base_url = "http://127.0.0.1:3000"
database_url = "sqlite://forkforge_dev.db"
api_timeout_seconds = 30

# GitHub OAuth
github_client_id = "your-github-client-id"
github_client_secret = "your-github-client-secret"

# Stripe Configuration
stripe_publishable_key = "pk_test_..."
stripe_secret_key = "sk_test_..."

[prod]
api_host = "0.0.0.0"
api_port = 8080
database_url = "postgres://forkforge:password@localhost/forkforge"
```

### Environment Variables

Override configuration with environment variables:

- `FORKFORGE_PROFILE` - Configuration profile (default: "default")
- `FORKFORGE_API_HOST` - API server host
- `FORKFORGE_API_PORT` - API server port
- `FORKFORGE_DATABASE_URL` - Database connection string
- `FORKFORGE_GITHUB_CLIENT_ID` - GitHub OAuth app ID
- `FORKFORGE_GITHUB_CLIENT_SECRET` - GitHub OAuth app secret
- `FORKFORGE_API_TIMEOUT_SECONDS` - API request timeout

## Development

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build --package domain
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test --package domain
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for security vulnerabilities
cargo audit
```

### Database Migrations

```bash
# Run migrations
cargo run --bin migrate

# Create new migration
sqlx migrate add <migration_name>
```

## Domain Services

### Authentication Service

- Clean architecture with `DeviceFlowProvider` trait in domain
- `GitHubDeviceFlowProvider` implementation in infrastructure
- Domain service (`AuthService`) orchestrates authentication flows
- Extensible design for additional providers (Google, etc.)
- Complete separation of business logic from OAuth implementation details

### Forking Service (Coming Soon)

- Solana validator forking
- Account cloning from mainnet
- RPC interactions

### Billing Service

- Stripe webhook processing
- Subscription management
- Usage tracking

### Snapshot Service

- Time-travel snapshots
- State persistence
- Snapshot sharing

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust naming conventions
- Use `cargo fmt` before committing
- Add tests for new functionality
- Update documentation as needed
- Keep business logic in the domain crate
- Use dependency injection for external services

## Roadmap

- [x] Domain-driven architecture setup
- [x] GitHub OAuth authentication
- [x] Basic API endpoints
- [ ] Solana validator forking implementation
- [ ] Account cloning from RPC
- [ ] Time-travel snapshot system
- [ ] Stripe billing integration
- [ ] Production deployment
- [ ] Kubernetes orchestration
- [ ] ZFS snapshot backend

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with Rust and love for the Solana ecosystem
- Powered by Axum, Tokio, and SQLx
- Inspired by the need for better local Solana development tools
