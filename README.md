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
│   └── domain/      # Core business logic and domain models
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

- **API Layer** (`crates/api/`): HTTP server implementation
  - Axum-based REST API
  - Adapters for domain services
  - Database and external service integrations

- **CLI Layer** (`crates/cli/`): Command-line interface
  - User interaction and display logic
  - API client implementation

- **Common Layer** (`crates/common/`): Shared components
  - Data Transfer Objects (DTOs)
  - Configuration management with Figment

### Key Features

- **Domain-Driven Design**: Business logic isolated in domain crate
- **Dependency Inversion**: Domain defines interfaces, infrastructure implements them
- **Profile-based Configuration**: Environment-specific settings via `config.toml`
- **GitHub OAuth Integration**: Device flow authentication
- **Extensible Service Architecture**: Easy to add new auth providers and services

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

- GitHub OAuth device flow implementation
- Extensible design for additional providers (Google, etc.)
- Token management and user authentication

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
