# dashboard アーキテクチャ

**構築フェーズ**: Phase 5

## 責務

- posts と post_metrics の可視化
- 型別・期間別の反応データ集計
- アカウントごとの運用状況把握
- メトリクス入力UIの提供（手動入力フェーズ）

## 非責務

- 情報収集(info-collector の責務）
- 投稿生成（post-generator の責務）
- 投稿作業（post-operator の責務）

## 構成

### バックエンド
- 本システムの Rust バックエンド（シングルバイナリ）の一部として dashboard API を提供
- Web サーバとして起動、ローカルで UI を提供
- SQLite への読み書きアクセス（UI からは API 経由のみ、直接 DB アクセスなし）

### フロントエンド
- React 等の任意フレームワーク（実装時に選定）
- バックエンドから API 経由でデータ取得
- ビルド成果物を Rust バイナリに埋め込む

## 提供機能

- 投稿一覧表示（アカウント・期間でフィルタ）
- 型別パフォーマンス分析
- 時系列推移表示
- メトリクス入力UI
- post_metrics の作成

## データアクセス

- posts / post_metrics / accounts などの読み取り
- post_metrics への書き込み（UI経由の手動入力）

## 判断を保留する実装論点

[../../decisions-pending.md](../../decisions-pending.md) 参照。
