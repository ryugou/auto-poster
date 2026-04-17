# Phase 0 基盤 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `auto-poster migrate` と `auto-poster seed` が動作し、CI が通る Rust ワークスペース基盤を構築する。

**Architecture:** cargo workspace に 6 crate（shared / info-collector / post-generator / post-operator / dashboard / cli）を配置。shared が DB・設定・ドメイン型・ログ・エラーを集約し、cli がサブコマンドランチャーとして各 feature crate を呼ぶ。Phase 0 では migrate / seed のみ実動作、他はスタブ。

**Tech Stack:** Rust 1.87+ / sqlx (SQLite, async, tokio) / figment (YAML + env) / clap (サブコマンド) / thiserror + anyhow / tracing / Docker (OrbStack)

**Design spec:** `docs/superpowers/specs/2026-04-17-phase0-foundation-design.md`

---

## File Structure

```
auto-poster/
├── Cargo.toml                          # workspace 定義
├── rust-toolchain.toml
├── .gitignore
├── .envrc
├── Makefile
├── compose.yaml
├── Dockerfile
├── .github/workflows/ci.yml
├── migrations/
│   └── 20260417000000_initial.sql
├── config/
│   ├── app.yaml
│   ├── accounts/
│   │   ├── ai_shinchaku.yaml
│   │   └── manga_shinkan.yaml
│   └── info_sources/
│       ├── grok.yaml
│       ├── rakuten_books.yaml
│       └── comic_natalie_rss.yaml
├── crates/
│   ├── shared/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── models.rs
│   │       ├── config.rs
│   │       ├── telemetry.rs
│   │       ├── prelude.rs
│   │       ├── testing.rs
│   │       └── db/
│   │           ├── mod.rs
│   │           ├── pool.rs
│   │           ├── account.rs
│   │           └── info_source.rs
│   ├── info-collector/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── post-generator/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── post-operator/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── dashboard/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── migrate.rs
│               ├── seed.rs
│               ├── collect.rs
│               ├── generate.rs
│               ├── operate.rs
│               └── serve.rs
└── ui/
    └── dist/
        └── .gitkeep
```

---

### Task 1: Workspace 骨格

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `crates/shared/Cargo.toml`
- Create: `crates/shared/src/lib.rs`
- Create: `crates/info-collector/Cargo.toml`
- Create: `crates/info-collector/src/lib.rs`
- Create: `crates/post-generator/Cargo.toml`
- Create: `crates/post-generator/src/lib.rs`
- Create: `crates/post-operator/Cargo.toml`
- Create: `crates/post-operator/src/lib.rs`
- Create: `crates/dashboard/Cargo.toml`
- Create: `crates/dashboard/src/lib.rs`
- Create: `crates/cli/Cargo.toml`
- Create: `crates/cli/src/main.rs`
- Create: `ui/dist/.gitkeep`

- [ ] **Step 1: rust-toolchain.toml と .gitignore を作成**

`rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.87.0"
components = ["rustfmt", "clippy"]
```

`.gitignore`:
```gitignore
/target
**/*.rs.bk
*.swp
*.swo
.env
*.db
*.db-shm
*.db-wal
.sqlx/
```

- [ ] **Step 2: workspace の Cargo.toml を作成**

`Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/info-collector",
    "crates/post-generator",
    "crates/post-operator",
    "crates/dashboard",
    "crates/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
figment = { version = "0.10", features = ["yaml", "env"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate", "chrono"] }
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 3: crates/shared/Cargo.toml と空 lib.rs を作成**

`crates/shared/Cargo.toml`:
```toml
[package]
name = "auto-poster-shared"
version.workspace = true
edition.workspace = true

[dependencies]
anyhow.workspace = true
chrono.workspace = true
figment.workspace = true
serde.workspace = true
serde_yaml.workspace = true
sqlx.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[features]
testing = []
```

`crates/shared/src/lib.rs`:
```rust
pub mod error;
```

- [ ] **Step 4: feature crate の Cargo.toml と空 lib.rs を作成**

`crates/info-collector/Cargo.toml`:
```toml
[package]
name = "auto-poster-info-collector"
version.workspace = true
edition.workspace = true

[dependencies]
auto-poster-shared = { path = "../shared" }
anyhow.workspace = true
sqlx.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/info-collector/src/lib.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn run(_pool: &SqlitePool, _account: Option<&str>) -> Result<()> {
    tracing::warn!("info-collector is not yet implemented");
    Ok(())
}
```

`crates/post-generator/Cargo.toml`:
```toml
[package]
name = "auto-poster-post-generator"
version.workspace = true
edition.workspace = true

[dependencies]
auto-poster-shared = { path = "../shared" }
anyhow.workspace = true
sqlx.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/post-generator/src/lib.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn run(_pool: &SqlitePool, _account: Option<&str>) -> Result<()> {
    tracing::warn!("post-generator is not yet implemented");
    Ok(())
}
```

`crates/post-operator/Cargo.toml`:
```toml
[package]
name = "auto-poster-post-operator"
version.workspace = true
edition.workspace = true

[dependencies]
auto-poster-shared = { path = "../shared" }
anyhow.workspace = true
sqlx.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/post-operator/src/lib.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn run_tui(_pool: &SqlitePool) -> Result<()> {
    tracing::warn!("post-operator is not yet implemented");
    Ok(())
}
```

`crates/dashboard/Cargo.toml`:
```toml
[package]
name = "auto-poster-dashboard"
version.workspace = true
edition.workspace = true

[dependencies]
auto-poster-shared = { path = "../shared" }
anyhow.workspace = true
sqlx.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/dashboard/src/lib.rs`:
```rust
use std::net::SocketAddr;

use anyhow::Result;
use sqlx::SqlitePool;

pub async fn serve(_pool: &SqlitePool, _addr: SocketAddr) -> Result<()> {
    tracing::warn!("dashboard is not yet implemented");
    Ok(())
}
```

- [ ] **Step 5: crates/cli/Cargo.toml と最小 main.rs を作成**

`crates/cli/Cargo.toml`:
```toml
[package]
name = "auto-poster"
version.workspace = true
edition.workspace = true

[[bin]]
name = "auto-poster"
path = "src/main.rs"

[dependencies]
auto-poster-shared = { path = "../shared" }
auto-poster-info-collector = { path = "../info-collector" }
auto-poster-post-generator = { path = "../post-generator" }
auto-poster-post-operator = { path = "../post-operator" }
auto-poster-dashboard = { path = "../dashboard" }
anyhow.workspace = true
clap.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/cli/src/main.rs`:
```rust
fn main() {
    println!("auto-poster: not yet implemented");
}
```

`ui/dist/.gitkeep`: (空ファイル)

- [ ] **Step 6: ビルドが通ることを確認**

Run: `cargo build`
Expected: 全 crate がエラーなしでコンパイル成功

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: cargo workspace 骨格を作成

6 crate（shared/info-collector/post-generator/post-operator/dashboard/cli）の
初期構成。feature crate はスタブ、shared は error モジュールのみ。"
```

---

### Task 2: shared::error + shared::prelude

**Files:**
- Create: `crates/shared/src/error.rs`
- Create: `crates/shared/src/prelude.rs`
- Modify: `crates/shared/src/lib.rs`

- [ ] **Step 1: error.rs を作成**

`crates/shared/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Config error: {0}")]
    Config(#[from] figment::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Info source not found: {0}")]
    InfoSourceNotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

- [ ] **Step 2: prelude.rs を作成**

`crates/shared/src/prelude.rs`:
```rust
pub use crate::error::{AppError, Result};
pub use tracing::{debug, error, info, warn};
```

- [ ] **Step 3: lib.rs を更新**

`crates/shared/src/lib.rs`:
```rust
pub mod error;
pub mod prelude;
```

- [ ] **Step 4: ビルド確認**

Run: `cargo build -p auto-poster-shared`
Expected: 成功

- [ ] **Step 5: Commit**

```bash
git add crates/shared/src/error.rs crates/shared/src/prelude.rs crates/shared/src/lib.rs
git commit -m "feat(shared): error 型と prelude モジュールを追加"
```

---

### Task 3: shared::models（ドメイン型）

**Files:**
- Create: `crates/shared/src/models.rs`
- Modify: `crates/shared/src/lib.rs`

- [ ] **Step 1: テストを先に書く**

`crates/shared/src/models.rs`（テスト部分を含む全体）:
```rust
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
```

- [ ] **Step 2: lib.rs に models を追加**

`crates/shared/src/lib.rs`:
```rust
pub mod error;
pub mod models;
pub mod prelude;
```

- [ ] **Step 3: テスト実行**

Run: `cargo test -p auto-poster-shared`
Expected: 4 tests passed

- [ ] **Step 4: Commit**

```bash
git add crates/shared/src/models.rs crates/shared/src/lib.rs
git commit -m "feat(shared): ドメインモデル型を追加"
```

---

### Task 4: DB プール初期化 + マイグレーション

**Files:**
- Create: `crates/shared/src/db/mod.rs`
- Create: `crates/shared/src/db/pool.rs`
- Create: `migrations/20260417000000_initial.sql`
- Modify: `crates/shared/src/lib.rs`

- [ ] **Step 1: マイグレーション SQL を作成**

`migrations/20260417000000_initial.sql`:
```sql
CREATE TABLE accounts (
    id         INTEGER PRIMARY KEY,
    yaml_key   TEXT NOT NULL UNIQUE,
    enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE info_sources (
    id          INTEGER PRIMARY KEY,
    yaml_key    TEXT NOT NULL UNIQUE,
    account_id  INTEGER NOT NULL REFERENCES accounts(id),
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE raw_materials (
    id            INTEGER PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id),
    source_id     INTEGER NOT NULL REFERENCES info_sources(id),
    natural_key   TEXT NOT NULL,
    title         TEXT NOT NULL,
    url           TEXT,
    summary       TEXT,
    raw_json      TEXT NOT NULL,
    metadata_json TEXT,
    status        TEXT NOT NULL DEFAULT 'unprocessed'
                  CHECK(status IN ('unprocessed','processed','filtered_out','error')),
    fetched_at    TEXT NOT NULL DEFAULT (datetime('now')),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id, natural_key)
);

CREATE TABLE drafts (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id),
    raw_material_id INTEGER NOT NULL REFERENCES raw_materials(id),
    template_type   TEXT NOT NULL CHECK(template_type IN ('T1','T2','T3','T4','T5')),
    body            TEXT NOT NULL,
    media_json      TEXT,
    scheduled_at    TEXT,
    review_status   TEXT NOT NULL DEFAULT 'pending'
                    CHECK(review_status IN ('pending','approved','rejected')),
    reviewed_at     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE posts (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id),
    draft_id        INTEGER NOT NULL REFERENCES drafts(id),
    template_type   TEXT NOT NULL CHECK(template_type IN ('T1','T2','T3','T4','T5')),
    body            TEXT NOT NULL,
    post_url        TEXT,
    scheduled_at    TEXT,
    posted_at       TEXT,
    recorded_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE post_metrics (
    id               INTEGER PRIMARY KEY,
    post_id          INTEGER NOT NULL REFERENCES posts(id),
    measured_at      TEXT NOT NULL DEFAULT (datetime('now')),
    impressions      INTEGER,
    likes            INTEGER,
    reposts          INTEGER,
    replies          INTEGER,
    bookmarks        INTEGER,
    profile_clicks   INTEGER,
    source           TEXT NOT NULL DEFAULT 'manual'
                     CHECK(source IN ('manual','x_api')),
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_raw_materials_account_status ON raw_materials(account_id, status);
CREATE INDEX idx_drafts_account_review ON drafts(account_id, review_status);
CREATE INDEX idx_posts_account ON posts(account_id);
CREATE INDEX idx_post_metrics_post ON post_metrics(post_id);
```

- [ ] **Step 2: db/pool.rs を作成**

`crates/shared/src/db/pool.rs`:
```rust
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
        .await
        .map_err(crate::error::AppError::Migration)?;
    Ok(())
}
```

- [ ] **Step 3: db/mod.rs を作成**

`crates/shared/src/db/mod.rs`:
```rust
mod pool;

pub use pool::{create_pool, run_migrations};
```

- [ ] **Step 4: lib.rs に db を追加**

`crates/shared/src/lib.rs`:
```rust
pub mod db;
pub mod error;
pub mod models;
pub mod prelude;
```

- [ ] **Step 5: テストを書く（in-memory DB でマイグレーション実行）**

`crates/shared/src/db/pool.rs` のテストモジュール末尾に追加:
```rust
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
```

- [ ] **Step 6: テスト実行**

Run: `cargo test -p auto-poster-shared`
Expected: 全テスト passed（models の 4 + db の 2 = 6）

- [ ] **Step 7: Commit**

```bash
git add migrations/ crates/shared/src/db/ crates/shared/src/lib.rs
git commit -m "feat(shared): DB プール初期化とマイグレーションを追加

sqlx::SqlitePool の生成ヘルパ（WAL モード、foreign_keys 有効）と
マイグレーションランナーを実装。初期スキーマで 6 テーブルを作成。"
```

---

### Task 5: DB リポジトリ関数（account / info_source）

**Files:**
- Create: `crates/shared/src/db/account.rs`
- Create: `crates/shared/src/db/info_source.rs`
- Modify: `crates/shared/src/db/mod.rs`

- [ ] **Step 1: account.rs のテストを先に書く**

`crates/shared/src/db/account.rs`:
```rust
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
    use crate::db::{create_pool, run_migrations};

    async fn setup() -> SqlitePool {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn upsert_creates_new_account() {
        let pool = setup().await;
        let id = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        assert!(id > 0);

        let account = find_by_yaml_key(&pool, "test_account").await.unwrap().unwrap();
        assert_eq!(account.yaml_key, "test_account");
        assert!(account.enabled);
    }

    #[tokio::test]
    async fn upsert_is_idempotent() {
        let pool = setup().await;
        let id1 = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        let id2 = upsert_by_yaml_key(&pool, "test_account").await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn list_enabled_filters_disabled() {
        let pool = setup().await;
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
```

- [ ] **Step 2: info_source.rs のテストを先に書く**

`crates/shared/src/db/info_source.rs`:
```rust
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
        ON CONFLICT(yaml_key) DO UPDATE SET updated_at = datetime('now')
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
    use crate::db::{self, create_pool, run_migrations};

    async fn setup() -> (SqlitePool, AccountId) {
        let pool = create_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let account_id = db::account::upsert_by_yaml_key(&pool, "test_account")
            .await
            .unwrap();
        (pool, account_id)
    }

    #[tokio::test]
    async fn upsert_creates_new_info_source() {
        let (pool, account_id) = setup().await;
        let id = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        assert!(id > 0);

        let source = find_by_yaml_key(&pool, "grok").await.unwrap().unwrap();
        assert_eq!(source.yaml_key, "grok");
        assert_eq!(source.account_id, account_id);
    }

    #[tokio::test]
    async fn upsert_is_idempotent() {
        let (pool, account_id) = setup().await;
        let id1 = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        let id2 = upsert_by_yaml_key(&pool, "grok", account_id).await.unwrap();
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn list_enabled_for_account_works() {
        let (pool, account_id) = setup().await;
        upsert_by_yaml_key(&pool, "grok", account_id)
            .await
            .unwrap();
        upsert_by_yaml_key(&pool, "rss", account_id)
            .await
            .unwrap();

        let sources = list_enabled_for_account(&pool, account_id).await.unwrap();
        assert_eq!(sources.len(), 2);
    }
}
```

- [ ] **Step 3: db/mod.rs を更新**

`crates/shared/src/db/mod.rs`:
```rust
pub mod account;
pub mod info_source;
mod pool;

pub use pool::{create_pool, run_migrations};
```

- [ ] **Step 4: テスト実行**

Run: `cargo test -p auto-poster-shared`
Expected: 全テスト passed（models 4 + db::pool 2 + db::account 3 + db::info_source 3 = 12）

- [ ] **Step 5: Commit**

```bash
git add crates/shared/src/db/
git commit -m "feat(shared): account / info_source リポジトリ関数を追加

upsert_by_yaml_key / find_by_yaml_key / list_enabled 系の
DB アクセス関数を実装。seed コマンドの基盤。"
```

---

### Task 6: shared::config（YAML + env ロード）

**Files:**
- Create: `crates/shared/src/config.rs`
- Create: `config/app.yaml`
- Create: `config/accounts/ai_shinchaku.yaml`
- Create: `config/accounts/manga_shinkan.yaml`
- Create: `config/info_sources/grok.yaml`
- Create: `config/info_sources/rakuten_books.yaml`
- Create: `config/info_sources/comic_natalie_rss.yaml`
- Modify: `crates/shared/src/lib.rs`

- [ ] **Step 1: config YAML ファイルを作成**

`config/app.yaml`:
```yaml
database_url: "sqlite:data/auto-poster.db"
log_level: "info"
log_format: "pretty"  # "pretty" or "json"
```

`config/accounts/ai_shinchaku.yaml`:
```yaml
key: ai_shinchaku
display_name: AI新着まとめ
handle: "@ai_shinchaku"
domain: ai_tools

schedule:
  posting_slots:
    - label: morning
      range: "07:00-09:00"
    - label: noon
      range: "12:00-13:00"
    - label: evening
      range: "21:00-23:00"
  info_collector_frequency: 3
  post_generator_frequency: 3

freshness_requirement: "数時間以内"

pipeline:
  step2_decompose:
    fields:
      - tool_name
      - provider
      - capabilities
      - pricing
      - release_date
      - buzz_score
  step3_filter:
    include_criteria:
      - "話題性が一定以上"
      - "一般ユーザーが使える"
      - "事実として確認可能"
    exclude_criteria:
      - "研究論文レベルの抽象的情報"
      - "エンジニア以外には価値がわからない技術的深掘り"
      - "事実確認が取れない情報"

ng_rules:
  - "有料プロモーションを示唆しない"
  - "特定ツールを過度にディスらない"
  - "AI生成であることを示唆する表現は避ける"
```

`config/accounts/manga_shinkan.yaml`:
```yaml
key: manga_shinkan
display_name: マンガ新刊速報
handle: "@manga_shinkan"
domain: manga

schedule:
  posting_slots:
    - label: morning
      range: "07:00-09:00"
    - label: noon
      range: "12:00-13:00"
    - label: evening
      range: "21:00-23:00"
  info_collector_frequency: 1
  post_generator_frequency: 3

freshness_requirement: "発売日当日内"

pipeline:
  step2_decompose:
    fields:
      - title
      - volume
      - author
      - publisher
      - release_date
      - price
      - ebook_available
      - sale_info
      - buzz_score
  step3_filter:
    include_criteria:
      - "話題性が一定以上"
      - "一般読者に知名度がある"
      - "事実として確認可能"
    exclude_criteria:
      - "同人誌・成人向け"
      - "特定読者層に限定されすぎる作品"
      - "発売日が未定・不確定な情報"

ng_rules:
  - "ネタバレ禁止"
  - "作者・作品への誹謗中傷禁止"
  - "海賊版サイトへの言及・リンク禁止"
```

`config/info_sources/grok.yaml`:
```yaml
key: grok
account_key: ai_shinchaku
source_type: api
display_name: xAI Grok API
```

`config/info_sources/rakuten_books.yaml`:
```yaml
key: rakuten_books
account_key: manga_shinkan
source_type: api
display_name: 楽天ブックスAPI
```

`config/info_sources/comic_natalie_rss.yaml`:
```yaml
key: comic_natalie_rss
account_key: manga_shinkan
source_type: rss
display_name: コミックナタリー RSS
```

- [ ] **Step 2: config.rs を作成（型定義 + ロード関数 + テスト）**

`crates/shared/src/config.rs`:
```rust
use std::path::Path;

use figment::providers::{Env, Format, Yaml};
use figment::Figment;
use serde::Deserialize;

use crate::error::Result;

// --- app.yaml ---

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

// --- accounts/*.yaml ---

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

// --- info_sources/*.yaml ---

#[derive(Debug, Clone, Deserialize)]
pub struct InfoSourceConfig {
    pub key: String,
    pub account_key: String,
    pub source_type: String,
    pub display_name: String,
}

// --- ロード関数 ---

pub fn load_app_config(config_dir: &Path) -> Result<AppConfig> {
    let config = Figment::new()
        .merge(Yaml::file(config_dir.join("app.yaml")))
        .merge(Env::prefixed("APP_"))
        .extract::<AppConfig>()?;
    Ok(config)
}

pub fn load_account_configs(config_dir: &Path) -> Result<Vec<AccountConfig>> {
    let accounts_dir = config_dir.join("accounts");
    let mut configs = Vec::new();

    if !accounts_dir.exists() {
        return Ok(configs);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&accounts_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        let config: AccountConfig = serde_yaml::from_str(&content)?;
        configs.push(config);
    }

    Ok(configs)
}

pub fn load_info_source_configs(config_dir: &Path) -> Result<Vec<InfoSourceConfig>> {
    let sources_dir = config_dir.join("info_sources");
    let mut configs = Vec::new();

    if !sources_dir.exists() {
        return Ok(configs);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&sources_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path())?;
        let config: InfoSourceConfig = serde_yaml::from_str(&content)?;
        configs.push(config);
    }

    Ok(configs)
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
```

- [ ] **Step 3: lib.rs に config を追加**

`crates/shared/src/lib.rs`:
```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod prelude;
```

- [ ] **Step 4: テスト実行**

Run: `cargo test -p auto-poster-shared`
Expected: 全テスト passed（models 4 + db 8 + config 6 = 18）

- [ ] **Step 5: Commit**

```bash
git add config/ crates/shared/src/config.rs crates/shared/src/lib.rs
git commit -m "feat(shared): config モジュールと YAML 設定ファイルを追加

figment で app.yaml + env を merge する AppConfig、
accounts/*.yaml / info_sources/*.yaml の型付きロードを実装。"
```

---

### Task 7: shared::telemetry + shared::testing

**Files:**
- Create: `crates/shared/src/telemetry.rs`
- Create: `crates/shared/src/testing.rs`
- Modify: `crates/shared/src/lib.rs`
- Modify: `crates/shared/src/prelude.rs`

- [ ] **Step 1: telemetry.rs を作成**

`crates/shared/src/telemetry.rs`:
```rust
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

pub fn init(log_level: &str, format: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    match format {
        "json" => {
            fmt()
                .with_env_filter(filter)
                .json()
                .init();
        }
        _ => {
            fmt()
                .with_env_filter(filter)
                .init();
        }
    }
}
```

- [ ] **Step 2: testing.rs を作成（testing feature gate 付き）**

`crates/shared/src/testing.rs`:
```rust
use sqlx::SqlitePool;

use crate::db;
use crate::error::Result;
use crate::models::AccountId;

/// テスト用の in-memory SQLite pool を作成しマイグレーション適用済みで返す。
pub async fn test_pool() -> SqlitePool {
    let pool = db::create_pool("sqlite::memory:").await.unwrap();
    db::run_migrations(&pool).await.unwrap();
    pool
}

/// テスト用のアカウントを seed して (pool, account_id) を返す。
pub async fn test_pool_with_account(yaml_key: &str) -> Result<(SqlitePool, AccountId)> {
    let pool = test_pool().await;
    let id = db::account::upsert_by_yaml_key(&pool, yaml_key).await?;
    Ok((pool, id))
}
```

- [ ] **Step 3: lib.rs を更新**

`crates/shared/src/lib.rs`:
```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod prelude;
pub mod telemetry;

#[cfg(any(test, feature = "testing"))]
pub mod testing;
```

- [ ] **Step 4: prelude.rs を更新**

`crates/shared/src/prelude.rs`:
```rust
pub use crate::error::{AppError, Result};
pub use tracing::{debug, error, info, warn};
```

- [ ] **Step 5: テスト実行**

Run: `cargo test -p auto-poster-shared`
Expected: 全テスト passed

- [ ] **Step 6: Commit**

```bash
git add crates/shared/src/telemetry.rs crates/shared/src/testing.rs crates/shared/src/lib.rs crates/shared/src/prelude.rs
git commit -m "feat(shared): telemetry 初期化と testing ヘルパを追加"
```

---

### Task 8: CLI ランチャー（clap サブコマンド + migrate / seed 実装）

**Files:**
- Create: `crates/cli/src/commands/mod.rs`
- Create: `crates/cli/src/commands/migrate.rs`
- Create: `crates/cli/src/commands/seed.rs`
- Create: `crates/cli/src/commands/collect.rs`
- Create: `crates/cli/src/commands/generate.rs`
- Create: `crates/cli/src/commands/operate.rs`
- Create: `crates/cli/src/commands/serve.rs`
- Modify: `crates/cli/src/main.rs`
- Modify: `crates/cli/Cargo.toml`

- [ ] **Step 1: cli の Cargo.toml を更新（figment 依存追加）**

`crates/cli/Cargo.toml`:
```toml
[package]
name = "auto-poster"
version.workspace = true
edition.workspace = true

[[bin]]
name = "auto-poster"
path = "src/main.rs"

[dependencies]
auto-poster-shared = { path = "../shared" }
auto-poster-info-collector = { path = "../info-collector" }
auto-poster-post-generator = { path = "../post-generator" }
auto-poster-post-operator = { path = "../post-operator" }
auto-poster-dashboard = { path = "../dashboard" }
anyhow.workspace = true
clap.workspace = true
tokio.workspace = true
tracing.workspace = true
```

- [ ] **Step 2: commands/mod.rs を作成（サブコマンド定義）**

`crates/cli/src/commands/mod.rs`:
```rust
pub mod collect;
pub mod generate;
pub mod migrate;
pub mod operate;
pub mod seed;
pub mod serve;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// SQLite マイグレーションを適用する
    Migrate,
    /// YAML 設定を DB に同期する
    Seed,
    /// 情報収集を実行する
    Collect {
        /// 対象アカウントの yaml_key（省略時は全有効アカウント）
        #[arg(long)]
        account: Option<String>,
    },
    /// 投稿ドラフトを生成する
    Generate {
        /// 対象アカウントの yaml_key（省略時は全有効アカウント）
        #[arg(long)]
        account: Option<String>,
    },
    /// レビュー TUI を起動する
    Operate,
    /// ダッシュボード API + UI サーバを起動する
    Serve {
        /// バインドアドレス
        #[arg(long, default_value = "0.0.0.0:3000")]
        addr: String,
    },
}
```

- [ ] **Step 3: commands/migrate.rs を作成**

`crates/cli/src/commands/migrate.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool) -> Result<()> {
    tracing::info!("Running database migrations...");
    auto_poster_shared::db::run_migrations(pool).await?;
    tracing::info!("Migrations applied successfully");
    Ok(())
}
```

- [ ] **Step 4: commands/seed.rs を作成**

`crates/cli/src/commands/seed.rs`:
```rust
use std::path::Path;

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use auto_poster_shared::config;
use auto_poster_shared::db;

pub async fn execute(pool: &SqlitePool, config_dir: &Path) -> Result<()> {
    tracing::info!("Seeding database from config files...");

    // accounts
    let account_configs = config::load_account_configs(config_dir)
        .context("Failed to load account configs")?;

    for ac in &account_configs {
        let id = db::account::upsert_by_yaml_key(pool, &ac.key).await?;
        tracing::info!(yaml_key = %ac.key, id = id, "Account seeded");
    }

    // info_sources
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
```

- [ ] **Step 5: スタブコマンドを作成**

`crates/cli/src/commands/collect.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool, account: Option<&str>) -> Result<()> {
    auto_poster_info_collector::run(pool, account).await?;
    Ok(())
}
```

`crates/cli/src/commands/generate.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool, account: Option<&str>) -> Result<()> {
    auto_poster_post_generator::run(pool, account).await?;
    Ok(())
}
```

`crates/cli/src/commands/operate.rs`:
```rust
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool) -> Result<()> {
    auto_poster_post_operator::run_tui(pool).await?;
    Ok(())
}
```

`crates/cli/src/commands/serve.rs`:
```rust
use std::net::SocketAddr;

use anyhow::Result;
use sqlx::SqlitePool;

pub async fn execute(pool: &SqlitePool, addr: &str) -> Result<()> {
    let addr: SocketAddr = addr.parse()?;
    auto_poster_dashboard::serve(pool, addr).await?;
    Ok(())
}
```

- [ ] **Step 6: main.rs を作成**

`crates/cli/src/main.rs`:
```rust
mod commands;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use auto_poster_shared::config;
use auto_poster_shared::db;
use auto_poster_shared::telemetry;

use commands::Commands;

#[derive(Debug, Parser)]
#[command(name = "auto-poster", about = "X 自動運用システム")]
struct Cli {
    /// 設定ディレクトリのパス
    #[arg(long, default_value = "config", global = true)]
    config_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let app_config = config::load_app_config(&cli.config_dir)?;
    telemetry::init(&app_config.log_level, &app_config.log_format);

    let pool = db::create_pool(&app_config.database_url).await?;

    match cli.command {
        Commands::Migrate => {
            commands::migrate::execute(&pool).await?;
        }
        Commands::Seed => {
            commands::seed::execute(&pool, &cli.config_dir).await?;
        }
        Commands::Collect { ref account } => {
            commands::collect::execute(&pool, account.as_deref()).await?;
        }
        Commands::Generate { ref account } => {
            commands::generate::execute(&pool, account.as_deref()).await?;
        }
        Commands::Operate => {
            commands::operate::execute(&pool).await?;
        }
        Commands::Serve { ref addr } => {
            commands::serve::execute(&pool, addr).await?;
        }
    }

    Ok(())
}
```

- [ ] **Step 7: ビルド確認**

Run: `cargo build`
Expected: 成功

- [ ] **Step 8: 動作確認（migrate + seed）**

Run:
```bash
mkdir -p data
cargo run -- migrate
cargo run -- seed
```

Expected:
- `data/auto-poster.db` が作成される
- migrate: "Migrations applied successfully"
- seed: "Account seeded" × 2, "Info source seeded" × 3, "Seed complete"

- [ ] **Step 9: seed の冪等性を確認（2回目も成功すること）**

Run: `cargo run -- seed`
Expected: 同じ出力、エラーなし

- [ ] **Step 10: Commit**

```bash
git add crates/cli/ data/.gitkeep
git commit -m "feat(cli): サブコマンドランチャーと migrate / seed を実装

clap による 6 サブコマンド（migrate/seed/collect/generate/operate/serve）。
migrate は sqlx マイグレーション適用、seed は YAML → DB upsert。
collect/generate/operate/serve は feature crate のスタブを呼ぶ。"
```

---

### Task 9: Docker + compose.yaml + Makefile

**Files:**
- Create: `Dockerfile`
- Create: `compose.yaml`
- Create: `Makefile`
- Create: `.envrc`

- [ ] **Step 1: Dockerfile を作成**

`Dockerfile`:
```dockerfile
# --- dev ---
FROM rust:1-bookworm AS dev
RUN cargo install cargo-watch sqlx-cli --locked
WORKDIR /app

# --- build ---
FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release

# --- release ---
FROM debian:bookworm-slim AS release
RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/auto-poster /usr/local/bin/
COPY --from=build /app/config /etc/auto-poster/config
COPY --from=build /app/migrations /etc/auto-poster/migrations
ENTRYPOINT ["auto-poster"]
CMD ["--help"]
```

- [ ] **Step 2: compose.yaml を作成**

`compose.yaml`:
```yaml
services:
  dashboard:
    build:
      context: .
      target: dev
    command: cargo watch -x "run -- serve"
    ports:
      - "3000:3000"
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/app/target
      - db-data:/app/data
    env_file:
      - path: .env
        required: false

  app:
    build:
      context: .
      target: dev
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/app/target
      - db-data:/app/data
    env_file:
      - path: .env
        required: false
    profiles: ["cli"]

volumes:
  cargo-cache:
  target-cache:
  db-data:
```

- [ ] **Step 3: Makefile を作成**

`Makefile`:
```makefile
COMPOSE := docker compose
RUN     := $(COMPOSE) run --rm app

.PHONY: help
help: ## ヘルプ表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'

# --- 開発 ---

.PHONY: dev
dev: ## dashboard + cargo watch を起動
	$(COMPOSE) up dashboard

.PHONY: build-release
build-release: ## release image をビルド
	docker build --target release -t auto-poster:latest .

# --- DB ---

.PHONY: migrate
migrate: ## マイグレーション適用
	$(RUN) auto-poster migrate

.PHONY: seed
seed: ## YAML → DB upsert
	$(RUN) auto-poster seed

# --- バッチ ---

.PHONY: collect
collect: ## 情報収集（ACCOUNT= で絞り込み可）
	$(RUN) auto-poster collect $(if $(ACCOUNT),--account $(ACCOUNT))

.PHONY: generate
generate: ## ドラフト生成（ACCOUNT= で絞り込み可）
	$(RUN) auto-poster generate $(if $(ACCOUNT),--account $(ACCOUNT))

.PHONY: operate
operate: ## レビュー TUI
	$(COMPOSE) run --rm -it app auto-poster operate

# --- テスト / lint ---

.PHONY: test
test: ## テスト実行
	$(RUN) cargo test --all

.PHONY: lint
lint: ## clippy
	$(RUN) cargo clippy --all-targets -- -D warnings

.PHONY: fmt
fmt: ## rustfmt チェック
	$(RUN) cargo fmt -- --check

.PHONY: sqlx-check
sqlx-check: ## sqlx prepare の整合チェック
	$(RUN) cargo sqlx prepare --check
```

- [ ] **Step 4: .envrc を作成**

`.envrc`:
```bash
# 1Password CLI からシークレットを取得する例（実際の vault パスに合わせて調整）
# export GROK_API_KEY="$(op read 'op://Private/grok-api/credential')"
# export RAKUTEN_APP_ID="$(op read 'op://Private/rakuten/app-id')"

# env override 例
# export APP_DATABASE_URL="sqlite:data/auto-poster.db"
# export APP_LOG_LEVEL="debug"
```

- [ ] **Step 5: Commit**

```bash
git add Dockerfile compose.yaml Makefile .envrc
git commit -m "feat: Docker / compose / Makefile / .envrc を追加

dev image（cargo-watch + sqlx-cli）と release image の multi-stage 構成。
compose は dashboard（常駐）+ app（ワンショット）の 2 サービス。
Makefile で全操作をラップ。"
```

---

### Task 10: CI ワークフロー

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: ci.yml を作成**

`.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy --all-targets

      - name: Run tests
        run: cargo test --all
```

- [ ] **Step 2: ローカルで CI 相当を実行して通ることを確認**

Run:
```bash
cargo fmt -- --check && cargo clippy --all-targets && cargo test --all
```

Expected: 全パス

- [ ] **Step 3: Commit**

```bash
git add .github/
git commit -m "ci: GitHub Actions ワークフローを追加

fmt / clippy / test を ubuntu-latest で実行。Swatinem/rust-cache でキャッシュ。"
```

---

### Task 11: 最終確認 + クリーンアップ

**Files:**
- 既存ファイルの確認のみ

- [ ] **Step 1: クリーンビルド確認**

Run: `cargo build --release`
Expected: 成功、`target/release/auto-poster` バイナリが生成される

- [ ] **Step 2: 全テスト実行**

Run: `cargo test --all`
Expected: 全テスト passed

- [ ] **Step 3: lint 確認**

Run: `cargo fmt -- --check && cargo clippy --all-targets`
Expected: 警告なし

- [ ] **Step 4: migrate + seed の E2E 確認**

Run:
```bash
rm -f data/auto-poster.db
cargo run -- migrate
cargo run -- seed
cargo run -- seed  # 冪等性確認
```

Expected: 2回目の seed もエラーなし

- [ ] **Step 5: サブコマンドのスタブ確認**

Run:
```bash
cargo run -- collect
cargo run -- generate
```

Expected: "not yet implemented" の warn ログが出て正常終了

- [ ] **Step 6: help 確認**

Run: `cargo run -- --help`
Expected: 6 サブコマンドが表示される

- [ ] **Step 7: 最終 Commit（もし未コミットの修正があれば）**

```bash
git status
# 未コミットの変更があれば add + commit
```

---

## Spec Coverage Check

| 設計 spec セクション | 対応タスク |
|---|---|
| 1. 配布モデル（シングルバイナリ、サブコマンド） | Task 1, 8 |
| 2. cargo workspace 構成（6 crate、依存方向） | Task 1, 2, 3, 4, 5, 6, 7, 8 |
| 3. 設定レイヤ（YAML + env + DB、合成フロー） | Task 6, 8 (seed) |
| 4. DB スキーマ（6 テーブル、インデックス） | Task 4 |
| 5. Docker / Makefile / CI | Task 9, 10 |
| 6. テスト戦略（in-memory SQLite、shared fixture） | Task 4, 5, 6, 7 |

Phase 0 ゴール:
- ✅ `auto-poster migrate` が動作する → Task 4, 8
- ✅ `auto-poster seed` が動作する → Task 5, 6, 8
- ✅ 他サブコマンドはスタブで存在する → Task 1, 8
- ✅ CI が通る → Task 10, 11
