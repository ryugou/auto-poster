use std::net::SocketAddr;

use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool, addr: &str) -> Result<()> {
    let addr: SocketAddr = addr.parse()?;
    auto_poster_dashboard::serve(pool, addr).await?;
    Ok(())
}
