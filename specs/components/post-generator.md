# post-generator

**構築フェーズ**: Phase 3

raw_materials から投稿ドラフトを生成し、drafts DB に保存するコンポーネント。

## 責務

- raw_materials の未処理素材の読み取り
- 5ステップパイプラインの実行
- drafts DB への保存
- raw_materials の処理ステータス更新
- 各ドラフトの投稿予定時刻の決定

## 非責務

- 情報収集（info-collector の責務）
- レビュー・投稿（post-operator の責務）

## 入力

- raw_materials DB の未処理レコード
- アカウント定義（[../operations/accounts.md](../operations/accounts.md)）
- 投稿型テンプレート（[../operations/post-templates.md](../operations/post-templates.md)）
- LLM の認証情報

## 出力

- drafts DB への新規レコード

## 5ステップパイプライン

各ステップの位置付け:

- **Step1 素材取得**: raw_materials の未処理分を取得
- **Step2 分解**: 素材を構造化要素に分解（アカウント固有。[../operations/accounts.md](../operations/accounts.md) 参照）
- **Step3 素材選別**: 投稿に値する素材を選別（アカウント固有。[../operations/accounts.md](../operations/accounts.md) 参照）
- **Step4 型選択**: 5型から選択（共通。[../operations/post-templates.md](../operations/post-templates.md) 参照）
- **Step5 ドラフト生成**: 選択した型のテンプレに沿って投稿文生成（共通）。生成時に投稿予定時刻も合わせて決定する。決定ロジックはアカウント定義の投稿時間帯と、素材の速報性を加味する。

## 処理の流れ

```
1. raw_materials から未処理素材を取得
2. 素材ごとに Step2 → Step3 → Step4 → Step5 を実行
3. 選別から漏れた場合: raw_materials ステータスを「選別除外」に更新
4. 素材の速報性とアカウントの投稿時間帯設定から投稿予定時刻を決定
5. 生成成功時: drafts に保存、raw_materials を「処理済」に更新
6. 処理結果をログ出力
```

## データライフサイクル管理

post-generator は drafts の作成を担当する。却下ドラフトの自動削除方針は post-operator 側の責務とする。

## エラーハンドリング

- LLM呼び出し失敗: 該当素材のステータスをエラーに、手動リトライ可能
- 型選択不能: 該当素材をスキップ、ログ記録
- DB保存失敗: ログ記録

## 判断を保留する実装論点

[../decisions-pending.md](../decisions-pending.md) 参照。
