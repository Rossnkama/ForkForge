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
- **Configuration Library**: Shared configuration module (forkforge-config) for centralized config management
- **Configuration**: Currently uses environment variables with plans to integrate figment for hierarchical configuration

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

The project uses environment variables for configuration. See `.env.example` for available options:

### Environment Variables

- `FORKFORGE_API_HOST` - API server host (default: "127.0.0.1")
- `FORKFORGE_API_PORT` - API server port (default: 3000)
- `FORKFORGE_API_BASE_URL` - Full API URL for CLI (default: "<http://127.0.0.1:3000>")
- `FORKFORGE_DATABASE_URL` - Database connection string (default: "sqlite://forkforge.db")
- `FORKFORGE_STRIPE_WEBHOOK_SECRET` - Stripe webhook secret for billing
- `FORKFORGE_API_TIMEOUT_SECONDS` - API request timeout (default: 30)

### Setup

1. Copy `.env.example` to `.env`:

   ```bash
   cp .env.example .env
   ```

2. Update the values in `.env` as needed

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
- [ ] Migrate from environment variables to figment hierarchical configuration
- [ ] Stripe webhook validation
- [ ] ZFS snapshot integration
- [ ] Kubernetes deployment

