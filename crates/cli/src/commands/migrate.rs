use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool) -> Result<()> {
    tracing::info!("Running database migrations...");
    auto_poster_shared::db::run_migrations(pool).await?;
    tracing::info!("Migrations applied successfully");
    Ok(())
}
