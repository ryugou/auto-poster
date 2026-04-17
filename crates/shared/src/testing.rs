use sqlx::SqlitePool;

use crate::db;
use crate::error::Result;
use crate::models::AccountId;

pub async fn test_pool() -> SqlitePool {
    let pool = db::create_pool("sqlite::memory:").await.unwrap();
    db::run_migrations(&pool).await.unwrap();
    pool
}

pub async fn test_pool_with_account(yaml_key: &str) -> Result<(SqlitePool, AccountId)> {
    let pool = test_pool().await;
    let id = db::account::upsert_by_yaml_key(&pool, yaml_key).await?;
    Ok((pool, id))
}
