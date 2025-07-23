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

### Quick Start - Super Simple Migration Runner (NEW)

We now have a dead-simple migration runner that's only 8 lines of code:

```bash
cd crates/api
cargo run --bin migrate
```

That's it! This command:

- Connects to the SQLite database (creates it if missing)
- Runs all pending migrations from the `migrations/` folder
- Only applies new migrations (idempotent - safe to run multiple times)

### How It Works

The migration runner (`crates/api/src/bin/migrate.rs`) is incredibly simple:

```rust
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./forkforge_dev.db?mode=rwc").await?;
    sqlx::migrate!("../migrations").run(&pool).await?;
    println!("✅ Migrations applied");
    Ok(())
}
```

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
