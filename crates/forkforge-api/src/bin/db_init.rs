use forkforge_config::Config;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ForkForge Database Initialization");

    // Load configuration
    let config = Config::load()?;
    println!(
        "📋 Loaded configuration for profile: {}",
        std::env::var("FORKFORGE_PROFILE").unwrap_or_else(|_| "default".to_string())
    );

    // Parse database URL and ensure it has the correct format for SQLite
    let db_url = if config.database_url.starts_with("sqlite:") {
        // Ensure we have the create mode flag
        if !config.database_url.contains("?mode=") {
            format!("{}?mode=rwc", config.database_url)
        } else {
            config.database_url.clone()
        }
    } else {
        return Err("Only SQLite databases are supported in this initialization tool".into());
    };

    println!("🗄️  Database URL: {}", db_url);

    // Create connection options with create_if_missing
    let connect_options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    // Create connection pool
    println!("🔌 Connecting to database...");
    let pool = SqlitePool::connect_with(connect_options).await?;

    // Run migrations
    println!("🔄 Running migrations...");
    let migrations_result = MIGRATOR.run(&pool).await;

    match migrations_result {
        Ok(_) => {
            println!("✅ Migrations completed successfully!");

            // Verify tables were created
            let tables: Vec<(String,)> = sqlx::query_as(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name"
            )
            .fetch_all(&pool)
            .await?;

            println!("\n📊 Created tables:");
            for (table_name,) in tables {
                println!("   - {}", table_name);
            }

            // Show migration history
            let migrations: Vec<(i64, String)> = sqlx::query_as(
                "SELECT version, description FROM _sqlx_migrations ORDER BY version",
            )
            .fetch_all(&pool)
            .await?;

            println!("\n📝 Applied migrations:");
            for (version, description) in migrations {
                println!("   - {} {}", version, description);
            }
        }
        Err(e) => {
            eprintln!("❌ Migration failed: {}", e);
            return Err(e.into());
        }
    }

    // Close the pool
    pool.close().await;

    println!("\n✨ Database initialization complete!");
    Ok(())
}
