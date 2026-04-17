use std::path::Path;

use figment::Figment;
use figment::providers::{Env, Format, Yaml};
use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountConfig {
    pub key: String,
    pub display_name: String,
    pub handle: String,
    pub domain: String,
    pub schedule: ScheduleConfig,
    pub freshness_requirement: String,
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub ng_rules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleConfig {
    pub posting_slots: Vec<PostingSlot>,
    pub info_collector_frequency: u32,
    pub post_generator_frequency: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostingSlot {
    pub label: String,
    pub range: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineConfig {
    pub step2_decompose: DecomposeConfig,
    pub step3_filter: FilterConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecomposeConfig {
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterConfig {
    pub include_criteria: Vec<String>,
    pub exclude_criteria: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InfoSourceConfig {
    pub key: String,
    pub account_key: String,
    pub source_type: String,
    pub display_name: String,
}

pub fn load_app_config(config_dir: &Path) -> Result<AppConfig> {
    let config = Figment::new()
        .merge(Yaml::file(config_dir.join("app.yaml")))
        .merge(Env::prefixed("APP_"))
        .extract::<AppConfig>()?;
    Ok(config)
}

fn load_yaml_dir<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    entries
        .into_iter()
        .map(|entry| {
            let content = std::fs::read_to_string(entry.path())?;
            Ok(serde_yaml::from_str(&content)?)
        })
        .collect()
}

pub fn load_account_configs(config_dir: &Path) -> Result<Vec<AccountConfig>> {
    load_yaml_dir(&config_dir.join("accounts"))
}

pub fn load_info_source_configs(config_dir: &Path) -> Result<Vec<InfoSourceConfig>> {
    load_yaml_dir(&config_dir.join("info_sources"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn config_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("config")
    }

    #[test]
    fn load_app_config_from_file() {
        let config = load_app_config(&config_dir()).unwrap();
        assert!(config.database_url.contains("sqlite:"));
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn load_account_configs_finds_two_accounts() {
        let configs = load_account_configs(&config_dir()).unwrap();
        assert_eq!(configs.len(), 2);

        let keys: Vec<&str> = configs.iter().map(|c| c.key.as_str()).collect();
        assert!(keys.contains(&"ai_shinchaku"));
        assert!(keys.contains(&"manga_shinkan"));
    }

    #[test]
    fn ai_shinchaku_config_has_three_posting_slots() {
        let configs = load_account_configs(&config_dir()).unwrap();
        let ai = configs.iter().find(|c| c.key == "ai_shinchaku").unwrap();
        assert_eq!(ai.schedule.posting_slots.len(), 3);
        assert_eq!(ai.schedule.info_collector_frequency, 3);
    }

    #[test]
    fn manga_shinkan_config_has_collector_frequency_one() {
        let configs = load_account_configs(&config_dir()).unwrap();
        let manga = configs.iter().find(|c| c.key == "manga_shinkan").unwrap();
        assert_eq!(manga.schedule.info_collector_frequency, 1);
        assert_eq!(manga.schedule.post_generator_frequency, 3);
    }

    #[test]
    fn load_info_source_configs_finds_three_sources() {
        let configs = load_info_source_configs(&config_dir()).unwrap();
        assert_eq!(configs.len(), 3);

        let keys: Vec<&str> = configs.iter().map(|c| c.key.as_str()).collect();
        assert!(keys.contains(&"grok"));
        assert!(keys.contains(&"rakuten_books"));
        assert!(keys.contains(&"comic_natalie_rss"));
    }

    #[test]
    fn info_source_references_valid_account_key() {
        let account_configs = load_account_configs(&config_dir()).unwrap();
        let source_configs = load_info_source_configs(&config_dir()).unwrap();

        let account_keys: Vec<&str> = account_configs.iter().map(|c| c.key.as_str()).collect();
        for source in &source_configs {
            assert!(
                account_keys.contains(&source.account_key.as_str()),
                "info_source '{}' references unknown account_key '{}'",
                source.key,
                source.account_key
            );
        }
    }
}
