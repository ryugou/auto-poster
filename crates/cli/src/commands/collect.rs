use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool, account: Option<&str>) -> Result<()> {
    auto_poster_info_collector::run(pool, account).await?;
    Ok(())
}
