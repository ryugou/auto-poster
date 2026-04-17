use sqlx::SqlitePool;

use crate::error::Result;
use crate::models::{Account, AccountId};

pub async fn upsert_by_yaml_key(pool: &SqlitePool, yaml_key: &str) -> Result<AccountId> {
    let result = sqlx::query_scalar::<_, AccountId>(
        r#"
        INSERT INTO accounts (yaml_key)
        VALUES (?)
        ON CONFLICT(yaml_key) DO UPDATE SET updated_at = datetime('now')
        RETURNING id
        "#,
    )
    .bind(yaml_key)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn find_by_yaml_key(pool: &SqlitePool, yaml_key: &str) -> Result<Option<Account>> {
    let row = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE yaml_key = ?")
        .bind(yaml_key)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_enabled(pool: &SqlitePool) -> Result<Vec<Account>> {
    let rows = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE enabled = TRUE")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[tokio::test]
    async fn upsert_creates_new_account() {
        let pool = testing::test_pool().await;
        let id = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        assert!(id > 0);

        let account = find_by_yaml_key(&pool, "test_account")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.yaml_key, "test_account");
        assert!(account.enabled);
    }

    #[tokio::test]
    async fn upsert_is_idempotent() {
        let pool = testing::test_pool().await;
        let id1 = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        let id2 = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn list_enabled_filters_disabled() {
        let pool = testing::test_pool().await;
        upsert_by_yaml_key(&pool, "enabled_one").await.unwrap();
        let id2 = upsert_by_yaml_key(&pool, "disabled_one").await.unwrap();
        sqlx::query("UPDATE accounts SET enabled = FALSE WHERE id = ?")
            .bind(id2)
            .execute(&pool)
            .await
            .unwrap();

        let enabled = list_enabled(&pool).await.unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].yaml_key, "enabled_one");
    }
}
