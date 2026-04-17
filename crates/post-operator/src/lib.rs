use anyhow::Result;
use sqlx::SqlitePool;

pub async fn run_tui(_pool: &SqlitePool) -> Result<()> {
    tracing::warn!("post-operator is not yet implemented");
    Ok(())
}
