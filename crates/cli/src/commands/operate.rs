use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool) -> Result<()> {
    auto_poster_post_operator::run_tui(pool).await?;
    Ok(())
}
