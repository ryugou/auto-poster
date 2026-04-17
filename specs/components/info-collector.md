# info-collector

**構築フェーズ**: Phase 2

外部情報源から素材を取得し、raw_materials DB に保存するコンポーネント。

## 責務

- 外部情報源からの情報取得
- レスポンスの共通スキーマへの正規化
- 重複検出
- raw_materials DB への保存

## 非責務

- 素材の品質評価（post-generator 側で実施）
- 投稿ドラフトの生成（post-generator の責務）
- エラー通知

## 入力

- 情報源の定義（[../operations/information-sources.md](../operations/information-sources.md) 参照）
- 情報源の認証情報（1Password CLI経由）

## 出力

- raw_materials DB への新規レコード

## 実行頻度

実行頻度はアカウントごとに異なる。[../operations/accounts.md](../operations/accounts.md) の各アカウント定義を参照。

## 処理の流れ

```
1. アカウントごとに情報源を取得
2. 各情報源から最新データを取得
3. 共通スキーマに正規化
4. raw_materials DB の既存データと重複検出
5. 新規分のみを raw_materials に保存
6. 処理結果をログ出力
```

情報源ごとの取得方針の詳細は [../operations/information-sources.md](../operations/information-sources.md) 参照。

## 共通スキーマ（raw_materials）

詳細は [dashboard/data-model.md](./dashboard/data-model.md) 参照。

## データライフサイクル管理

info-collector は raw_materials の作成を担当する。古い未処理素材の削除方針は実装判断とする。

## エラーハンドリング

- 情報源取得失敗: ログ記録、次回実行に委ねる
- レスポンス形式不正: ログ記録、該当情報源をスキップ
- DB保存失敗: ログ記録、リトライはしない

## 判断を保留する実装論点

[../decisions-pending.md](../decisions-pending.md) 参照。
