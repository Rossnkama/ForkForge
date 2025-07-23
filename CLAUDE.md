# ForkForge (Chainbox) Project

Fast Solana mainnet forking CLI for local development and testing.

## Core Instructions

### Do ONLY What's Asked

- **NEVER** add features beyond the exact request
- **NEVER** create files unless absolutely necessary
- **ALWAYS** prefer editing existing files over creating new ones
- **ONLY** implement the specific task - nothing more, nothing less

### Code Quality

- Write idiomatic Rust with minimal abstractions
- Favor simplicity over cleverness
- Keep diff sizes small
- Use existing patterns in the codebase

### Before File Operations

- Run `pwd` before any mkdir/cd commands
- Verify directory location before creating/modifying files
- Use absolute paths when uncertain

## Project Overview

- **Purpose**: Create state-accurate Solana mainnet forks in <10s with account cloning, time-travel snapshots, and integrated debugging
- **Architecture**: Rust CLI (cli) + API server (api) with future cloud layer for snapshots
- **Key Feature**: Clone only required accounts from mainnet, not entire 100TB+ state

## Project Business Description

### Chainbox/Forkforge

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

## Commands

- `cargo build`: Build all crates
- `cargo run --bin api`: Start API server (127.0.0.1:3000)
- `cargo run --bin cli -- up`: Launch forked validator
- `cargo test`: Run tests
- `cargo fmt`: Format code
- `cargo clippy`: Run linter

## Code Style

- Rust 2024 edition
- `tokio` for async
- `figment` for config
- Explicit `Result` error handling
- Standard Rust naming conventions

## Learning Preferences

- Personal learning notes go in `personal_learning.md` (gitignored) - keep them information-dense with annotated `rust` code examples showing real production patterns. Explain with beauty like Richard Feynmann

- If I ask for a brief explanation of a pattern or anything else, don't add to personal learning unless specifically instructed to do so. Always explain like Richard Feynmann, as well as concisely with lots of relevant examples that are made up and (if applicable) from the codebase.

## Configuration

- Config: `config.toml` at project root
- Profiles: default, prod (via `FORKFORGE_PROFILE`)
- Env vars: `FORKFORGE_` prefix overrides config

## Workflow Rules

- Run `cargo fmt` and `cargo clippy` after code changes
- Test only what changed, not entire suite
- Create feature branches from develop
- Check existing patterns before implementing new ones
