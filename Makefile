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
