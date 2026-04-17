# システム全体像

## 目的

定点観測型Xアカウントを複数並行運用し、情報キュレーションでフォロワー獲得を実現する。横展開（将来的に10アカウント規模）を前提とした共通基盤を構築する。

## 設計原則

### 1. 領域非依存の共通基盤 + 領域固有の設定分離

情報収集・投稿生成・運用のロジックは領域非依存で実装。各アカウント固有の設定（情報源、キーワード、Step2-3の具体内容、ペルソナ）は設定ファイルで差分管理する。

新規アカウント追加時のコストを最小化することで、10アカウント規模への横展開を現実的にする。

### 2. 5ステップの投稿生成パイプライン

投稿は以下のステップで生成される。各ステップの品質を独立に評価・改善できる構造を取る。

```
Step1: トレンド・素材取得
Step2: 分解 (アカウント固有)
Step3: 素材選別 (アカウント固有)
Step4: 型選択 (共通)
Step5: ドラフト生成 (共通)
```

- Step2-3 はアカウントの領域によって分解要素・選別基準が異なるため、アカウント定義で規定する
- Step4-5 は5型テンプレを全アカウント共通で使用する
- アカウント固有定義: [operations/accounts.md](./operations/accounts.md)
- 共通の型テンプレ: [operations/post-templates.md](./operations/post-templates.md)

### 3. 手動フェーズから始める

立ち上げ初期は手動投稿。自動化は型が固まってから導入する。改善ループが回らない段階での自動化は、筋の悪い出力を量産するだけで意味がない。

### 4. 評価は各ステップ独立に

事前評価（投稿前チェックリスト）と事後評価（反応データ）を両方持つ。どのステップの品質が低いかを特定できる構造にする。

### 5. Progressive Disclosure

本spec含め全ドキュメントは、トップを薄く、詳細を下位ファイルに分離する構成を徹底する。

## システム境界

### システム内
- 情報収集パイプライン
- 投稿生成ロジック
- 投稿管理（ドラフト保管、レビュー記録、投稿記録）
- 分析ダッシュボード

### システム外
- X本体への投稿アクション
- 外部情報源
- X Analytics

## 配布形態

全バックエンド機能とダッシュボード UI を Rust 製シングルバイナリに集約して配布する。ダッシュボード UI は無くても自動投稿運用が完結することを要件とする。

## 主要コンポーネント

| コンポーネント | 責務 | 詳細 |
|---|---|---|
| info-collector | 外部情報源から素材取得、正規化、raw_materials DBへ保存 | [components/info-collector.md](./components/info-collector.md) |
| post-generator | raw_materials → 5ステップパイプラインで drafts DB へ投稿ドラフト生成 | [components/post-generator.md](./components/post-generator.md) |
| post-operator | drafts のレビュー・投稿記録を受け取り、posts DB へ保存 | [components/post-operator.md](./components/post-operator.md) |
| dashboard | posts DB と反応データを可視化、メトリクス入力UIを提供、型別評価を提供 | [components/dashboard/](./components/dashboard/) |

## データフロー

詳細: [architecture/data-flow.md](./architecture/data-flow.md)

## 運用対象アカウント

| アカウント | 領域 | 情報源 | 投稿頻度 |
|---|---|---|---|
| @ai_shinchaku | AI新着 | Grok API | 日1-3投稿 |
| @manga_shinkan | マンガ新刊 | 楽天ブックスAPI、コミックナタリーRSS | 日1-3投稿 |

詳細: [operations/accounts.md](./operations/accounts.md)

投稿スケジュール詳細: [operations/posting-schedule.md](./operations/posting-schedule.md)

## 実装ロードマップ

詳細: [roadmap.md](./roadmap.md)

## 判断を保留する実装論点

システム全体を通じた未決事項は [decisions-pending.md](./decisions-pending.md) に集約。
