# 投稿スケジュール

## 投稿時間帯

各アカウントの投稿時間帯は [accounts.md](./accounts.md) で定義する。

ターゲット層の活動時間に合わせ、以下の3枠を基本とする:
- 朝枠: 7-9時（通勤・起床直後）
- 昼枠: 12-13時（昼休み）
- 夜枠: 21-23時（一日の終わり）

## 1日のタイムライン（標準例）

```
06:30  info-collector (朝枠用素材取得)
06:45  post-generator (朝枠ドラフト生成)
07:00  Ryugo レビュー → 手動投稿
11:30  info-collector (昼枠用素材取得、A のみ)
11:45  post-generator (昼枠ドラフト生成)
12:00  Ryugo レビュー → 手動投稿
20:30  info-collector (夜枠用素材取得、A のみ)
20:45  post-generator (夜枠ドラフト生成)
21:00  Ryugo レビュー → 手動投稿
```

アカウントBは朝の info-collector のみ実行し、post-generator は3回走る。

## 投稿予定時刻の決定

post-generator がドラフト生成時に決定する。決定ロジック:

- ドラフトが生成されたタイミングの直後の投稿時間帯を割り当てる
- 素材の速報性が高い場合は直近枠、低い場合は後続枠に調整

## 手動投稿の運用

- Xアプリで人間が投稿
- 投稿後、投稿URLを post-operator に記録
- スケジュール投稿ツール（SocialDog等）の利用可否は実装判断

## 判断を保留する実装論点

- スケジュール投稿ツールの採用可否
- 投稿時間帯の調整権限（post-generator の決定を post-operator で上書き可能とするか）
- 休日・祝日のスケジュール調整

[../decisions-pending.md](../decisions-pending.md) 参照。
