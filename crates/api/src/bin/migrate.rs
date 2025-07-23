use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:./forkforge_dev.db?mode=rwc").await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    println!("cargo:rerun-if-changed=migrations");
    println!("✅ Script ran");
    Ok(())
}
