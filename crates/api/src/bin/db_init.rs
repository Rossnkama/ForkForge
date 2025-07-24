use common::Config;

// Re-export from infra crate
pub use infra::MIGRATOR;
pub use infra::db::{SqlitePool, init_db, list_migrations, list_tables};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ForkForge Database Initialization");

    // Load configuration
    let config = Config::load()?;
    println!(
        "📋 Loaded configuration for profile: {}",
        std::env::var("FORKFORGE_PROFILE").unwrap_or_else(|_| "default".to_string())
    );

    println!("🗄️  Database URL: {}", config.database_url);

    // Initialize database and run migrations
    println!("🔌 Connecting to database...");
    println!("🔄 Running migrations...");
    let pool = init_db(&config.database_url).await?;

    println!("✅ Migrations completed successfully!");

    // Verify tables were created
    let tables = list_tables(&pool).await?;
    println!("\n📊 Created tables:");
    for table_name in tables {
        println!("   - {table_name}");
    }

    // Show migration history
    let migrations = list_migrations(&pool).await?;
    println!("\n📝 Applied migrations:");
    for (version, description) in migrations {
        println!("   - {version} {description}");
    }

    // Close the pool
    pool.close().await;

    println!("\n✨ Database initialization complete!");
    Ok(())
}
