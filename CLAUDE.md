# ForkForge (Chainbox) Project

Fast Solana mainnet forking CLI for local development and testing.

## Notes to claude

### Check working directory before relative location commands w.r.t the file system

When you want to run relative commands i.e. cd thing/... or mkdir, use pwd to make sure that you're in the directory that you think you're in so that you don't make or remove files in the wrong directory.

## Project Overview

- **Purpose**: Create state-accurate Solana mainnet forks in <10s with account cloning, time-travel snapshots, and integrated debugging
- **Architecture**: Rust CLI (forkforge-cli) + API server (forkforge-api) with future cloud layer for snapshots
- **Key Feature**: Clone only required accounts from mainnet, not entire 100TB+ state

## Project Business Description

### Chainbox

Chainbox is an open-source Rust CLI that lets a developer create a fully
forked, state-accurate Solana mainnet validator in **< 10 s** with
`chainbox up`, auto-cloning only the accounts listed in `sandbox.toml`,
pre-air-dropping wallets, and streaming color-coded logs. Above the CLI sits
a **proprietary cloud layer**—Kubernetes pods backed by ZFS snapshots—that
provides sharable URLs, one-click **time-travel snapshots**, a marketplace of
pre-indexed state packs, and usage-based billing (Stripe) with bundled
**Helius RPC credits**. The moat comes from the closed snapshot registry +
data network effects, SSPL licensing for the time-travel engine, and CI/IDE
integrations that embed immutable snapshot IDs directly into developer
workflows.

#### Example User Flow

1. **Alice (protocol engineer)** runs `chainbox up` in a repo that contains

   ```toml
   rpc_url  = "https://helius.xyz/<key>"
   clone_accounts = ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"]
   ```

   → Chainbox launches a forked validator in 8 s and airdrops 2 SOL to her dev wallet.

2. She hot-patches the SPL-Token program, redeploys with `chainbox deploy target/token.so`, and triggers a test transaction.

3. The program panics; she saves the exact state with `chainbox snapshot save "panic-2245"` which returns
   `https://app.chainbox.dev/snap/solana/panic-2245`.

4. **Bob (teammate)** clicks the link, Chainbox CLI auto-pulls the snapshot, and in <10 s his local validator is in the identical state/slot.

5. Bob attaches the Solana debugger, finds an un-initialised var, patches it, and pushes a PR—both confirm the fix by re-running the same snapshot ID in their CI pipeline.

This concise block can be supplied as meta-context to any LLM for technical, product, or go-to-market tasks.

## Project Structure

```
forkforge/
├── forkforge-api/      # Axum-based API server
│   ├── src/
│   │   └── main.rs    # API endpoints and server setup
├── forkforge-cli/      # Rust CLI tool
│   ├── src/
│   │   └── main.rs    # CLI commands and client
├── forkforge-config/   # Shared configuration library (NEW)
│   ├── src/
│   │   └── lib.rs     # Config struct and environment loading
├── forkforge-bruno/    # API testing collection
├── .env.example        # Example environment variables (NEW)
└── Cargo.toml         # Workspace root with shared dependencies
```

## Common Commands

### Development

- `cargo build`: Build both projects
- `cargo run --bin forkforge-api`: Start API server on 127.0.0.1:3000
- `cargo run --bin forkforge-cli -- up`: Launch forked validator (makes API call to /sessions)
- `cargo test`: Run tests
- `cargo fmt`: Format code
- `cargo clippy`: Run linter

### API Testing

- Bruno collection available in `forkforge-bruno/`
- Base URL: `http://127.0.0.1:3000`

## Code Style

- Use Rust 2024 edition features
- Prefer `tokio` for async runtime
- Use `figment` for configuration management
- Follow standard Rust naming conventions
- Prefer explicit error handling with `Result`

## Configuration

### Current State

The project currently uses environment variables through the `forkforge-config` shared library. The configuration is centralized in a single `Config` struct with `from_env()` and `Default` implementations.

### Environment Variables

All configuration is done through environment variables with `FORKFORGE_` prefix:

- `FORKFORGE_API_HOST` - API server host (default: "127.0.0.1")
- `FORKFORGE_API_PORT` - API server port (default: 3000)
- `FORKFORGE_DATABASE_URL` - Database connection string (default: "sqlite://forkforge.db")
- `FORKFORGE_STRIPE_WEBHOOK_SECRET` - Stripe webhook secret (default: empty)
- `FORKFORGE_API_TIMEOUT_SECONDS` - API request timeout (default: 30)

### forkforge-config Library Structure

```rust
Config {
    api_host: String,
    api_port: u16,
    database_url: String,
    stripe_webhook_secret: String,
    api_timeout_seconds: u64,
}
```

### Setup Instructions

1. Copy `.env.example` to `.env`
2. Update values as needed
3. The `forkforge-config` library will read these on initialization

### Future Plans

- Migration to figment for hierarchical configuration (TOML + env vars)
- Integration of `forkforge-config` into both API and CLI projects

## Key APIs

### forkforge-api endpoints

- `GET /health`: Health check - returns `{"data": "Ok"}`
- `POST /sessions`: Create new fork session (stub)
- `POST /snapshots/:id`: Create snapshot with ID parameter (stub)
- `POST /billing/webhook`: Stripe webhook handler (stub)

### Shared Workspace Dependencies

The root `Cargo.toml` defines workspace-level dependencies:

```toml
[workspace.dependencies]
tokio = { version = "1.46.1", features = ["rt", "full", "macros"] }
serde = { version = "1.0.219", features = ["derive"] }
figment = { version = "0.10.19", features = ["toml", "env"] }
```

Both projects reference these with `workspace = true`.

## Testing Strategy

- Unit tests for core logic
- Integration tests for API endpoints
- Snapshot testing for validator state

## Important Notes

- **Database**: SQLx with SQLite for local development
- **Authentication**: Stripe webhook validation implemented (HMAC-SHA256)
- **State Management**: Future ZFS snapshot integration planned
- **Performance**: Target <10s fork creation time
- **Configuration**: Currently using environment variables via `forkforge-config` library, with plans to migrate to figment
- **Async Runtime**: Tokio with full features across both projects
- **Error Handling**: Explicit `Result` types throughout
- **New Addition**: `forkforge-config` library centralizes configuration across API and CLI (not yet integrated)

## TODO Integration

- Solana validator fork implementation
- Account cloning from RPC
- Time-travel snapshot system
- CLI command implementation beyond `up`
- Kubernetes deployment configuration

## Dependencies to Know

- `axum`: Web framework for API
- `clap`: CLI argument parsing
- `figment`: Configuration management
- `sqlx`: Database access (SQLite)
- `reqwest`: HTTP client
- `tokio`: Async runtime

## Future Features

- Pre-indexed state packs marketplace
- Sharable snapshot URLs
- CI/IDE integrations
- Helius RPC credit bundling

## Recent Changes

- Added `forkforge-config` library as a new workspace member for centralized configuration
- Created `.env.example` file with all available environment variables
- Simplified configuration to use environment variables directly (temporary solution)
- Workspace still includes figment dependency for future hierarchical configuration
- Updated CLI description in Cargo.toml to reflect Chainbox functionality
- Config library provides `from_env()` and `Default` implementations

## Note on Configuration Status

The project is in transition:

- Previously attempted figment integration (references remain in workspace)
- Currently using simple environment variables via `forkforge-config`
- API and CLI still have their own configuration code (not yet using shared library)
- Plan to complete migration to shared config library, then add figment support
