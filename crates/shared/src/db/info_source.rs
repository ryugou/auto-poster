use sqlx::SqlitePool;

use crate::error::Result;
use crate::models::{AccountId, InfoSource, InfoSourceId};

pub async fn upsert_by_yaml_key(
    pool: &SqlitePool,
    yaml_key: &str,
    account_id: AccountId,
) -> Result<InfoSourceId> {
    let result = sqlx::query_scalar::<_, InfoSourceId>(
        r#"
        INSERT INTO info_sources (yaml_key, account_id)
        VALUES (?, ?)
        ON CONFLICT(yaml_key) DO UPDATE SET account_id = excluded.account_id, updated_at = datetime('now')
        RETURNING id
        "#,
    )
    .bind(yaml_key)
    .bind(account_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn find_by_yaml_key(pool: &SqlitePool, yaml_key: &str) -> Result<Option<InfoSource>> {
    let row = sqlx::query_as::<_, InfoSource>("SELECT * FROM info_sources WHERE yaml_key = ?")
        .bind(yaml_key)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_enabled_for_account(
    pool: &SqlitePool,
    account_id: AccountId,
) -> Result<Vec<InfoSource>> {
    let rows = sqlx::query_as::<_, InfoSource>(
        "SELECT * FROM info_sources WHERE account_id = ? AND enabled = TRUE",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[tokio::test]
    async fn upsert_creates_new_info_source() {
        let (pool, account_id) = testing::test_pool_with_account("test_account")
            .await
            .unwrap();
        let id = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        assert!(id > 0);

        let source = find_by_yaml_key(&pool, "grok").await.unwrap().unwrap();
        assert_eq!(source.yaml_key, "grok");
        assert_eq!(source.account_id, account_id);
    }

    #[tokio::test]
    async fn upsert_is_idempotent() {
        let (pool, account_id) = testing::test_pool_with_account("test_account")
            .await
            .unwrap();
        let id1 = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        let id2 = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn list_enabled_for_account_works() {
        let (pool, account_id) = testing::test_pool_with_account("test_account")
            .await
            .unwrap();
        upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        upsert_by_yaml_key(&pool, "rss", account_id).await.unwrap();

        let sources = list_enabled_for_account(&pool, account_id).await.unwrap();
        assert_eq!(sources.len(), 2);
    }
}
