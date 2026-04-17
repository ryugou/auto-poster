use std::path::Path;

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use auto_poster_shared::config;
use auto_poster_shared::db;

pub async fn execute(pool: &SqlitePool, config_dir: &Path) -> Result<()> {
    tracing::info!("Seeding database from config files...");

    let account_configs = config::load_account_configs(config_dir)
        .context("Failed to load account configs")?;

    for ac in &account_configs {
        let id = db::account::upsert_by_yaml_key(pool, &ac.key).await?;
        tracing::info!(yaml_key = %ac.key, id = id, "Account seeded");
    }

    let source_configs = config::load_info_source_configs(config_dir)
        .context("Failed to load info source configs")?;

    for sc in &source_configs {
        let account = db::account::find_by_yaml_key(pool, &sc.account_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Info source '{}' references unknown account '{}'",
                    sc.key,
                    sc.account_key
                )
            })?;

        let id = db::info_source::upsert_by_yaml_key(pool, &sc.key, account.id).await?;
        tracing::info!(yaml_key = %sc.key, account = %sc.account_key, id = id, "Info source seeded");
    }

    tracing::info!(
        accounts = account_configs.len(),
        info_sources = source_configs.len(),
        "Seed complete"
    );

    Ok(())
}
