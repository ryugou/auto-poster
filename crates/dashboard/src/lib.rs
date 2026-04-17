use std::net::SocketAddr;

use anyhow::Result;
use sqlx::SqlitePool;

pub async fn serve(_pool: &SqlitePool, _addr: SocketAddr) -> Result<()> {
    tracing::warn!("dashboard is not yet implemented");
    Ok(())
}
