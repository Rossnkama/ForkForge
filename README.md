# ForkForge (Chainbox)

Fast Solana mainnet forking CLI for local development and testing.

## Overview

ForkForge is an open-source Rust CLI that creates state-accurate Solana mainnet forks in <10s with account cloning, time-travel snapshots, and integrated debugging. Clone only required accounts from mainnet, not the entire 100TB+ state.

## Project Structure

```
forkforge/
├── forkforge-api/      # Axum-based API server
├── forkforge-cli/      # Rust CLI tool
├── forkforge-config/   # Shared configuration library
├── forkforge-bruno/    # API testing collection
├── .env.example        # Example environment variables
└── Cargo.toml          # Workspace root with shared dependencies
```

## Architecture

- **Workspace Structure**: Rust workspace with shared dependencies (tokio, serde, figment)
- **API Server**: Axum-based REST API for session management, snapshots, and billing
- **CLI Tool**: Command-line interface for launching forked validators
- **Configuration Library**: Shared configuration module (forkforge-config) with figment-based hierarchical configuration
- **Configuration**: Profile-based configuration via `config.toml` with environment variable overrides

## Getting Started

### Prerequisites

- Rust 1.75+ (2024 edition)
- Cargo

### Running the API Server

```bash
cargo run --bin forkforge-api
```

Starts HTTP server on `http://127.0.0.1:3000`

Available endpoints:

- `GET /health` - Health check
- `POST /sessions` - Create new fork session
- `POST /snapshots/:id` - Create snapshot
- `POST /billing/webhook` - Stripe webhook

### Running the CLI

```bash
cargo run --bin forkforge-cli -- up
```

## Configuration

The project uses figment for hierarchical configuration supporting both TOML files and environment variables.

### Configuration Profiles

Edit `config.toml` to define configuration profiles:

```toml
[default]
api_host = "127.0.0.1"
api_port = 3000
database_url = "sqlite://forkforge_dev.db"
api_timeout_seconds = 30

[prod]
api_host = "0.0.0.0"
api_port = 8080
database_url = "postgres://forkforge:password@localhost/forkforge"
api_timeout_seconds = 60
```

### Environment Variables

All configuration values can be overridden with environment variables:

- `FORKFORGE_PROFILE` - Select configuration profile (default: "default")
- `FORKFORGE_API_HOST` - API server host
- `FORKFORGE_API_PORT` - API server port
- `FORKFORGE_DATABASE_URL` - Database connection string
- `FORKFORGE_STRIPE_WEBHOOK_SECRET` - Stripe webhook secret for billing
- `FORKFORGE_API_TIMEOUT_SECONDS` - API request timeout

### Setup

1. Edit `config.toml` to define your configuration profiles

2. Optionally copy `.env.example` to `.env` for environment overrides:

   ```bash
   cp .env.example .env
   ```

3. Set the profile with `FORKFORGE_PROFILE=prod` to use production settings

### Configuration Precedence

1. Default values from code
2. Profile-specific values from `config.toml`
3. Environment variables (highest priority)

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Format & Lint

```bash
cargo fmt
cargo clippy
```

### Watch Mode

For development with auto-rebuild:

```bash
# Watch and run the API server
cargo watch -x "run --bin forkforge-api"

# Watch and run the CLI
cargo watch -x "run --bin forkforge-cli"
```

## Shared Dependencies

The workspace manages common dependencies:

- `tokio` (v1.46.1) - Async runtime with full features
- `serde` (v1.0.219) - Serialization framework
- `figment` (v0.10.19) - Configuration management

## API Testing

Bruno collection available in `forkforge-bruno/` for testing API endpoints.

## TODO

- [ ] Implement Solana validator forking
- [ ] Account cloning from RPC
- [ ] Time-travel snapshot system
- [ ] Complete CLI commands beyond `up`
- [ ] Integrate forkforge-config library into API and CLI
- [ ] Stripe webhook validation
- [ ] ZFS snapshot integration
- [ ] Kubernetes deployment
