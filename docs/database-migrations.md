# Database Migrations Guide

<!--toc:start-->
- [Database Migrations Guide](#database-migrations-guide)
  - [Running SQLx Migrations with SQLite](#running-sqlx-migrations-with-sqlite)
    - [Quick Start - Super Simple Migration Runner (NEW)](#quick-start-super-simple-migration-runner-new)
    - [How It Works](#how-it-works)
    - [Creating New Migrations](#creating-new-migrations)
    - [Recent Changes to Schema](#recent-changes-to-schema)
    - [Migration Files](#migration-files)
    - [Manual Migration Commands (Alternative)](#manual-migration-commands-alternative)
    - [Legacy Database Initialization Tool](#legacy-database-initialization-tool)
    - [Key SQLite Gotchas](#key-sqlite-gotchas)
    - [Verifying Migration Status](#verifying-migration-status)
    - [Best Practices](#best-practices)
<!--toc:end-->

## Running SQLx Migrations with SQLite

### Quick Start - Migration Management

All migrations are managed through a single `MIGRATOR` constant defined in `crates/infra/src/db.rs`. This ensures consistent migration paths across all binaries.

```bash
# Simple migration runner
cargo run --bin migrate

# Or use the detailed initialization tool
cargo run --bin db-init
```

Both commands:

- Use the same `infra::MIGRATOR` constant
- Connect to the SQLite database (create if missing)
- Run all pending migrations from the `migrations/` folder
- Are idempotent (safe to run multiple times)

### How It Works

The migration system uses a centralized approach:

1. **Single MIGRATOR Source**: Defined in `crates/infra/src/db.rs`:
   ```rust
   pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");
   ```

2. **Migration Runner** (`crates/api/src/bin/migrate.rs`):
   ```rust
   use infra::db::init_db;
   
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       let pool = init_db("sqlite:./forkforge_dev.db?mode=rwc").await?;
       pool.close().await;
       println!("✅ Script ran");
       Ok(())
   }
   ```

3. **DB Init Tool** (`crates/api/src/bin/db_init.rs`):
   - Uses `infra::MIGRATOR`
   - Provides detailed output about tables and migrations

### Creating New Migrations

1. Add a new SQL file to the `migrations/` directory with timestamp prefix:

   ```
   migrations/YYYYMMDD_NNNNNN_description.sql
   ```

   Example: `20250716_000003_add_github_auth.sql`

2. Write your SQL commands in the file

3. Run migrations:

   ```bash
   cargo run --bin migrate
   ```

### Recent Changes to Schema

We've added GitHub authentication support with the following changes:

1. **Users table**: Added `github_id` column
   - Column is nullable to support existing users
   - Has a unique index to prevent duplicate GitHub accounts

2. **Auth credentials table**: Added `last_used_at` timestamp
   - Tracks when tokens are actively used
   - `expires_at` was already nullable (for long-lived tokens)

### Migration Files

Current migrations in order:

1. `20250113_000001_init_schema.sql` - Initial schema with users, auth_credentials, and stripe_events tables
2. `20250716_000003_add_github_auth.sql` - Adds GitHub authentication fields

Note: We removed a duplicate `20250716_000002_init_schema.sql` that was accidentally trying to recreate the same tables.

### Manual Migration Commands (Alternative)

If you prefer using SQLx CLI directly:

```bash
# Install SQLx CLI if needed
cargo install sqlx-cli

# Run migrations
export DATABASE_URL="sqlite:./forkforge_dev.db?mode=rwc"
sqlx migrate run
```

### Legacy Database Initialization Tool

We still have the more complex `db-init` tool that shows detailed information:

```bash
cargo run --bin db-init
```

This tool:

- Loads configuration from `config.toml` or environment variables
- Shows created tables and migration history
- Provides verbose output

However, for day-to-day use, the simple `migrate` command is recommended.

### Key SQLite Gotchas

1. **SQLite URL Format**: Always use `?mode=rwc` to allow database creation
   - `r` = read, `w` = write, `c` = create if missing

2. **ALTER TABLE Limitations**: SQLite doesn't support all ALTER TABLE operations
   - Can't add UNIQUE constraint directly (use CREATE UNIQUE INDEX instead)
   - Can't modify column constraints (would need table recreation)

3. **Partial Index Restrictions**: Can't use `CURRENT_TIMESTAMP` in WHERE clauses

### Verifying Migration Status

```bash
# Check which migrations have been applied
sqlite3 forkforge_dev.db "SELECT version, description FROM _sqlx_migrations;"

# View table schemas
sqlite3 forkforge_dev.db ".schema users"
sqlite3 forkforge_dev.db ".schema auth_credentials"
```

### Best Practices

1. **Idempotent Migrations**: The migration system tracks applied migrations, so it's safe to run `cargo run --bin migrate` multiple times
2. **Test Locally First**: Always test migrations on your local database before committing
3. **Incremental Changes**: Each migration should be a small, focused change
4. **No Rollbacks**: SQLx doesn't support rollbacks - plan migrations carefully
