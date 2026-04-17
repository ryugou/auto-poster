use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// --- ID 型 ---

pub type AccountId = i64;
pub type InfoSourceId = i64;
pub type RawMaterialId = i64;
pub type DraftId = i64;
pub type PostId = i64;
pub type PostMetricId = i64;

// --- Enum ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum RawMaterialStatus {
    Unprocessed,
    Processed,
    FilteredOut,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
pub enum TemplateType {
    T1,
    T2,
    T3,
    T4,
    T5,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum MetricsSource {
    Manual,
    XApi,
}

// --- Row 型 ---

#[derive(Debug, Clone, FromRow)]
pub struct Account {
    pub id: AccountId,
    pub yaml_key: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct InfoSource {
    pub id: InfoSourceId,
    pub yaml_key: String,
    pub account_id: AccountId,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct RawMaterial {
    pub id: RawMaterialId,
    pub account_id: AccountId,
    pub source_id: InfoSourceId,
    pub natural_key: String,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub raw_json: String,
    pub metadata_json: Option<String>,
    pub status: RawMaterialStatus,
    pub fetched_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Draft {
    pub id: DraftId,
    pub account_id: AccountId,
    pub raw_material_id: RawMaterialId,
    pub template_type: TemplateType,
    pub body: String,
    pub media_json: Option<String>,
    pub scheduled_at: Option<NaiveDateTime>,
    pub review_status: ReviewStatus,
    pub reviewed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Post {
    pub id: PostId,
    pub account_id: AccountId,
    pub draft_id: DraftId,
    pub template_type: TemplateType,
    pub body: String,
    pub post_url: Option<String>,
    pub scheduled_at: Option<NaiveDateTime>,
    pub posted_at: Option<NaiveDateTime>,
    pub recorded_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct PostMetric {
    pub id: PostMetricId,
    pub post_id: PostId,
    pub measured_at: NaiveDateTime,
    pub impressions: Option<i64>,
    pub likes: Option<i64>,
    pub reposts: Option<i64>,
    pub replies: Option<i64>,
    pub bookmarks: Option<i64>,
    pub profile_clicks: Option<i64>,
    pub source: MetricsSource,
    pub created_at: NaiveDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_material_status_default_is_unprocessed() {
        let status = RawMaterialStatus::Unprocessed;
        assert_eq!(status, RawMaterialStatus::Unprocessed);
    }

    #[test]
    fn template_type_variants_exist() {
        let types = vec![
            TemplateType::T1,
            TemplateType::T2,
            TemplateType::T3,
            TemplateType::T4,
            TemplateType::T5,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn review_status_variants_exist() {
        let statuses = vec![
            ReviewStatus::Pending,
            ReviewStatus::Approved,
            ReviewStatus::Rejected,
        ];
        assert_eq!(statuses.len(), 3);
    }

    #[test]
    fn metrics_source_variants_exist() {
        let sources = vec![MetricsSource::Manual, MetricsSource::XApi];
        assert_eq!(sources.len(), 2);
    }
}
