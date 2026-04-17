# 技術スタック

## バックエンド

### Rust
- 用途: バックエンド全機能の実装
  - info-collector（情報収集）
  - post-generator（投稿生成）
  - post-operator（投稿オペレーション）
  - メトリクス取得
  - dashboard API
- ダッシュボード UI を含めた **シングルバイナリ配布** を前提とする
- 具体的なフレームワーク・ライブラリ選定は実装担当者が行う

## ダッシュボード UI

- ビルド成果物を Rust バイナリに埋め込む方針
- フレームワークは任意（React / その他）。実装時に選定
- UI からは Rust バックエンドの API のみを呼ぶ。DB への直接アクセスは行わない

## データベース

- SQLite 単一ファイル
- マイグレーション管理を行う
- スキーマ: [../components/dashboard/data-model.md](../components/dashboard/data-model.md)

## 開発環境

- Docker（OrbStack）で全サービス実行
- ローカルへの直接インストール禁止
- ランタイム管理にmise等のツールを使用

## シークレット管理

- 1Password CLI でシークレット取得
- direnv で環境変数として注入
- コミット対象にシークレットを含めない

## バージョン管理

- Git
- Conventional Commits（日本語）

## 外部サービス

### LLM
- 投稿ドラフト生成用のLLMを使用
- 具体サービス（Claude / Grok 等）は実装フェーズで選定

### X関連
- X Developer Portal の既存登録状況は未確認
- Phase 6 以降に X API 連携を想定
- 新規登録の場合、従量課金制のため初期チャージが必要

### データ取得
- AI側: xAI Grok API（トレンド取得）
- マンガ側: 楽天ブックスAPI、コミックナタリーRSS
- Amazon PA-API は Phase 6 以降の追加候補

## 禁止事項

- ローカルへの直接インストール（Docker内のみ）
- Python の使用（明示的に必要な場合を除く）
- 秘密情報のコミット
- 環境依存の絶対パスの使用
- ダッシュボード UI から DB への直接アクセス

## 判断を保留する実装論点

[../decisions-pending.md](../decisions-pending.md) 参照。
