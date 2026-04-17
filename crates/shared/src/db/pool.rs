use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::time::Duration;

use crate::error::Result;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_pool_in_memory() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn run_migrations_creates_tables() {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();

        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_sqlx_migrations' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
        assert!(names.contains(&"accounts"));
        assert!(names.contains(&"info_sources"));
        assert!(names.contains(&"raw_materials"));
        assert!(names.contains(&"drafts"));
        assert!(names.contains(&"posts"));
        assert!(names.contains(&"post_metrics"));
    }
}
