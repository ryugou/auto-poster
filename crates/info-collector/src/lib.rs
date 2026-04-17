use anyhow::Result;
use sqlx::SqlitePool;

pub async fn run(_pool: &SqlitePool, _account: Option<&str>) -> Result<()> {
    tracing::warn!("info-collector is not yet implemented");
    Ok(())
}
