use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use auto_poster_shared::config;
use auto_poster_shared::db;
use auto_poster_shared::models::AccountId;

pub async fn execute(pool: &SqlitePool, config_dir: &Path) -> Result<()> {
    tracing::info!("Seeding database from config files...");

    let account_configs =
        config::load_account_configs(config_dir).context("Failed to load account configs")?;

    let mut account_id_map: HashMap<String, AccountId> =
        HashMap::with_capacity(account_configs.len());

    for ac in &account_configs {
        let id = db::account::upsert_by_yaml_key(pool, &ac.key).await?;
        account_id_map.insert(ac.key.clone(), id);
        tracing::info!(yaml_key = %ac.key, id = id, "Account seeded");
    }

    let source_configs = config::load_info_source_configs(config_dir)
        .context("Failed to load info source configs")?;

    for sc in &source_configs {
        let account_id = account_id_map.get(&sc.account_key).ok_or_else(|| {
            anyhow::anyhow!(
                "Info source '{}' references unknown account '{}'",
                sc.key,
                sc.account_key
            )
        })?;

        let id = db::info_source::upsert_by_yaml_key(pool, &sc.key, *account_id).await?;
        tracing::info!(yaml_key = %sc.key, account = %sc.account_key, id = id, "Info source seeded");
    }

    tracing::info!(
        accounts = account_configs.len(),
        info_sources = source_configs.len(),
        "Seed complete"
    );

    Ok(())
}
