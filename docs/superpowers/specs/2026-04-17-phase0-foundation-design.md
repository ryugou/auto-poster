# Phase 0 基盤設計

## 概要

X 自動運用システム（auto-poster）の全 Phase を支える基盤を構築する。Phase 2 以降の全コンポーネント（info-collector / post-generator / post-operator / dashboard / metrics 取得）が乗る土台として、Rust ワークスペース・DB スキーマ・設定レイヤ・Docker 構成・CI を整備する。

## ゴール

- `auto-poster` という単一の Rust バイナリから全機能をサブコマンドで呼び分けられる構成
- `auto-poster migrate` と `auto-poster seed` が動作する状態
- `collect` / `generate` / `operate` / `serve` はスタブ（`todo!()` or 空の success）で crate 骨格のみ
- CI が通る状態（fmt / clippy / sqlx prepare --check / test）

## スコープ外

- 情報源からの実データ取得（Phase 2）
- ドラフト生成ロジック・LLM 連携（Phase 3）
- レビュー TUI の UI 実装（Phase 4）
- ダッシュボード UI（Phase 5）
- 自動投稿・スケジューラ（Phase 6）

---

## 1. 配布モデル

Rust 製の **シングルバイナリ** として配布する。ダッシュボード UI（React 等）のビルド成果物をこのバイナリに embed するが、UI が存在しなくても自動投稿運用は完結する。

### サブコマンド体系

| サブコマンド | 動作 | Phase |
|---|---|---|
| `auto-poster migrate` | SQLite マイグレーション適用 | 0 |
| `auto-poster seed` | YAML → accounts/info_sources を DB へ upsert | 0 |
| `auto-poster collect [--account <key>]` | 情報収集 | 2 |
| `auto-poster generate [--account <key>]` | ドラフト生成 | 3 |
| `auto-poster operate` | レビュー TUI | 4 |
| `auto-poster serve` | dashboard API + 埋め込み UI 常駐 | 5 |

`--account` 省略時は有効な全アカウントが対象。

### Docker 上の実行モデル

| モード | 呼び出し方 | compose サービス |
|---|---|---|
| 常駐（dashboard） | `auto-poster serve` | `dashboard`（`up -d`） |
| ワンショット | `auto-poster <subcommand>` | `app`（`run --rm`） |

---

## 2. cargo workspace 構成

### リポジトリレイアウト

```
auto-poster/
├── Cargo.toml                 # workspace 定義
├── rust-toolchain.toml        # Rust バージョン固定
├── Makefile
├── compose.yaml
├── Dockerfile                 # multi-stage（dev / build / release）
├── .envrc
├── .github/workflows/ci.yml
├── migrations/                # sqlx マイグレーション（NNN_*.sql）
├── config/
│   ├── app.yaml
│   ├── accounts/
│   │   ├── ai_shinchaku.yaml
│   │   └── manga_shinkan.yaml
│   └── info_sources/
│       ├── grok.yaml
│       ├── rakuten_books.yaml
│       └── comic_natalie_rss.yaml
├── ui/                        # ダッシュボード UI（Phase 5 で追加、Phase 0 は空）
│   └── dist/
├── crates/
│   ├── shared/
│   ├── info-collector/
│   ├── post-generator/
│   ├── post-operator/
│   ├── dashboard/
│   └── cli/
├── specs/
└── docs/
```

### crate 責務

**`crates/shared`** — バックエンド標準ライブラリ

| モジュール | 責務 |
|---|---|
| `models` | ドメイン型（`Account`, `RawMaterial`, `Draft`, `Post`, `PostMetrics`, `InfoSource`） |
| `db` | `SqlitePool` 初期化 + リポジトリ関数群 |
| `config` | YAML + env ロード（`figment`）、DB 突合で合成 `Account` / `InfoSource` を返す |
| `telemetry` | `tracing` 初期化（dev=pretty, prod=JSON, stdout 出力） |
| `error` | `thiserror` ベースの crate 横断 Error 型 |
| `testing` | in-memory pool ファクトリ、共通 fixture（`#[cfg(feature = "testing")]`） |
| `prelude` | よく使う型の re-export |

**`crates/info-collector`** — 情報収集

- `pub async fn run(cfg, pool, account?) -> Result<Summary>`
- 情報源アダプタ（Grok API / 楽天ブックス API / RSS）を内包
- 正規化 → `shared::db::raw_material::insert_if_not_exists`

**`crates/post-generator`** — 投稿生成

- `pub async fn run(cfg, pool, account?) -> Result<Summary>`
- 5ステップパイプライン
- LLM クライアントをこの crate に閉じる（他 crate が使い始めたら shared に昇格）
- 投稿予定時刻の決定ロジック

**`crates/post-operator`** — 投稿オペレーション

- `pub async fn run_tui(cfg, pool) -> Result<()>`
- 対話 TUI（`ratatui` 等）

**`crates/dashboard`** — ダッシュボード API + UI

- `pub async fn serve(cfg, pool, addr) -> Result<()>`
- HTTP サーバ（axum 想定）
- `rust-embed` で `ui/dist` を取り込み

**`crates/cli`** — 配布用バイナリ（薄いランチャー）

- `clap` でサブコマンド定義 → 対応 crate の `run` を呼ぶだけ
- バイナリ名: `auto-poster`

### 依存方向

```
cli
 ├── shared
 ├── info-collector ── shared
 ├── post-generator ── shared
 ├── post-operator  ── shared
 └── dashboard      ── shared
```

feature crate 同士は依存しない。

---

## 3. 設定レイヤ

### 3層構造

| レイヤ | 格納先 | 変更手段 | 内容 |
|---|---|---|---|
| YAML | `config/` (git管理) | PR レビュー | プロンプト、投稿時間帯、Step2/3 定義、NG ルール、API パラメータ |
| 環境変数 | `.envrc` (direnv + 1Password CLI) | direnv reload | API キー、DB パス、ログレベル |
| DB | SQLite `accounts` / `info_sources` | seed / 将来の dashboard UI | 有効フラグ、運用中に調整する閾値 |

### 起動時の合成フロー

1. `figment` が `config/app.yaml` + env → `AppConfig` に deserialize
2. `config/accounts/*.yaml` → `Vec<AccountYaml>`
3. `config/info_sources/*.yaml` → `Vec<InfoSourceYaml>`
4. DB 接続 + マイグレーション確認
5. `accounts` テーブルの `yaml_key` で突合 → 合成 `Account` 型
6. `info_sources` テーブルも同様
7. `AppContext { config, pool, accounts, info_sources }` を feature crate に渡す

### YAML 構造例（`config/accounts/ai_shinchaku.yaml`）

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

---

## 4. DB スキーマ

sqlx + sqlx-cli によるマイグレーション管理。マイグレーションファイルは `migrations/NNN_*.sql`。

### 初期スキーマ（`migrations/001_initial.sql`）

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

### 設計判断

- 日時は TEXT（ISO 8601）。SQLite の datetime 関数と互換、Rust 側は `chrono` or `time` に deserialize
- `natural_key`: 重複検出キー。情報源ごとに何を使うかは Phase 2 で確定
- status / review_status は TEXT + CHECK 制約。Rust 側は `sqlx::Type` derive で enum 対応
- JSON カラムは TEXT。初期は保存用、必要時に `json()` 関数で検索
- 外部キーの ON DELETE は未指定。自動カスケードは将来 migration で追加
- Phase 0 で全テーブルを作る（`seed` / `migrate` が一発で動く状態を確保）

---

## 5. Docker / Makefile / CI 構成

### compose.yaml

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
    env_file: .env

  app:
    build:
      context: .
      target: dev
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/app/target
      - db-data:/app/data
    env_file: .env
    profiles: ["cli"]

volumes:
  cargo-cache:
  target-cache:
  db-data:
```

### Dockerfile（multi-stage）

```dockerfile
FROM rust:1-bookworm AS dev
RUN cargo install cargo-watch sqlx-cli --locked
WORKDIR /app

FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS release
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/auto-poster /usr/local/bin/
COPY --from=build /app/config /etc/auto-poster/config
COPY --from=build /app/migrations /etc/auto-poster/migrations
ENTRYPOINT ["auto-poster"]
```

### Makefile

```makefile
COMPOSE = docker compose
RUN     = $(COMPOSE) run --rm app

.PHONY: dev
dev:
	$(COMPOSE) up dashboard

.PHONY: build-release
build-release:
	docker build --target release -t auto-poster:latest .

.PHONY: migrate
migrate:
	$(RUN) auto-poster migrate

.PHONY: seed
seed:
	$(RUN) auto-poster seed

.PHONY: seed-dev
seed-dev:
	$(RUN) auto-poster seed --dev

.PHONY: collect
collect:
	$(RUN) auto-poster collect $(if $(ACCOUNT),--account $(ACCOUNT))

.PHONY: generate
generate:
	$(RUN) auto-poster generate $(if $(ACCOUNT),--account $(ACCOUNT))

.PHONY: operate
operate:
	$(COMPOSE) run --rm -it app auto-poster operate

.PHONY: test
test:
	$(RUN) cargo test --all

.PHONY: lint
lint:
	$(RUN) cargo clippy --all-targets -- -D warnings

.PHONY: fmt
fmt:
	$(RUN) cargo fmt -- --check

.PHONY: sqlx-check
sqlx-check:
	$(RUN) cargo sqlx prepare --check
```

### CI（.github/workflows/ci.yml）

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo sqlx prepare --check
      - run: cargo test --all
```

### rust-toolchain.toml

```toml
[toolchain]
channel = "1.87.0"
components = ["rustfmt", "clippy"]
```

---

## 6. テスト戦略

- unit tests: `#[cfg(test)]` で各モジュールに inline
- integration tests: 各 crate の `tests/` に配置、DB に触る
- テストごとに in-memory SQLite（`sqlite::memory:`）を起動、`sqlx::migrate!()` で初期化
- 共通 fixture: `crates/shared/src/testing.rs`（`#[cfg(feature = "testing")]`）に pool ファクトリと基本 seed

---

## ブレスト時の決定事項まとめ

| 論点 | 決定 |
|---|---|
| ブレスト対象スコープ | Phase 0 基盤のみ |
| 実行環境 | ローカル完結、後で移設可能な構成 |
| 配布形態 | Rust シングルバイナリ、UI embed |
| プロジェクト構成 | cargo workspace + ランチャー crate |
| DB スタック | sqlx + sqlx-cli（async, tokio） |
| 設定レイヤ | YAML + env + DB ハイブリッド（figment） |
| Docker 構成 | compose 2サービス（dashboard 常駐 + app ワンショット）+ Makefile ラッパー |
| shared crate | 厚い shared（models / db / config / telemetry / error / testing / prelude） |
| スケジューラ | Phase 4 まで手動起動（make ターゲット）、Phase 6 で自動化 |
| リポ公開 | Public（CI minutes 無制限のため） |
| テスト | in-memory SQLite + shared fixture |
| CI | fmt / clippy / sqlx prepare --check / test（GitHub Actions、Rust キャッシュ） |
