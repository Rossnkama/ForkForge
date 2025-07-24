// Re-export from infra crate
pub use infra::db::init_db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db("sqlite:./forkforge_dev.db?mode=rwc").await?;
    pool.close().await;
    println!("cargo:rerun-if-changed=migrations");
    println!("✅ Script ran");
    Ok(())
}
