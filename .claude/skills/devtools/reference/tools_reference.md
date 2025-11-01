# Chrome DevTools ツールリファレンス

Chrome DevTools MCPサーバーが提供する全ツールの詳細リファレンス。

## 目次

- [ページ管理](#ページ管理)
- [ナビゲーション](#ナビゲーション)
- [コンテンツ取得](#コンテンツ取得)
- [要素操作](#要素操作)
- [デバッグ](#デバッグ)
- [パフォーマンス](#パフォーマンス)
- [エミュレーション](#エミュレーション)

---

## ページ管理

### list_pages

開いているページの一覧を取得。

```typescript
mcp__chrome-devtools__list_pages()
```

**レスポンス例：**
```
0: http://localhost:5173/dashboard [selected]
1: http://localhost:3000/login
```

### new_page

新しいページを作成して開く。

```typescript
mcp__chrome-devtools__new_page({
  url: "http://localhost:5173",
  timeout: 30000  // オプション、デフォルト30秒
})
```

### select_page

指定したページを選択（アクティブに）する。

```typescript
mcp__chrome-devtools__select_page({
  pageIdx: 1
})
```

### close_page

指定したページを閉じる。最後のページは閉じられない。

```typescript
mcp__chrome-devtools__close_page({
  pageIdx: 1
})
```

### resize_page

ページのウィンドウサイズを変更。

```typescript
mcp__chrome-devtools__resize_page({
  width: 1920,
  height: 1080
})
```

**用途：** レスポンシブデザインのテスト

---

## ナビゲーション

### navigate_page

現在選択中のページを指定URLに遷移。

```typescript
mcp__chrome-devtools__navigate_page({
  url: "http://localhost:5173/processes",
  timeout: 30000  // オプション
})
```

### navigate_page_history

ブラウザの履歴を使って前後に移動。

```typescript
mcp__chrome-devtools__navigate_page_history({
  navigate: "back",  // または "forward"
  timeout: 30000
})
```

---

## コンテンツ取得

### take_snapshot

アクセシビリティツリーからページコンテンツのテキスト表現を取得。

```typescript
mcp__chrome-devtools__take_snapshot({
  verbose: false  // オプション、詳細情報を含めるか
})
```

**レスポンス例：**
```
uid=1_0 RootWebArea "Vantage MCP"
  uid=1_1 banner
    uid=1_2 link "Vantage"
      uid=1_3 StaticText "Vantage"
    uid=1_4 navigation
      uid=1_5 link "ダッシュボード"
        uid=1_6 StaticText "ダッシュボード"
```

**重要：** UIDは要素操作に必須。スナップショットから取得する。

### take_screenshot

ページまたは特定要素のスクリーンショットを取得。

```typescript
// ページ全体
mcp__chrome-devtools__take_screenshot({
  fullPage: true,
  format: "png",  // "png", "jpeg", "webp"
  quality: 90,    // JPEG/WebPの品質（0-100）
  filePath: "/path/to/save.png"  // オプション、保存先
})

// 特定要素のみ
mcp__chrome-devtools__take_screenshot({
  uid: "1_7",
  format: "png"
})
```

---

## 要素操作

### click

要素をクリック。

```typescript
mcp__chrome-devtools__click({
  uid: "1_7",
  dblClick: false  // オプション、ダブルクリックの場合true
})
```

### hover

要素にマウスをホバー。

```typescript
mcp__chrome-devtools__hover({
  uid: "1_13"
})
```

### fill

フォーム要素に値を入力。

```typescript
mcp__chrome-devtools__fill({
  uid: "2_10",
  value: "test@example.com"
})
```

**対応要素：** input, textarea, select

### fill_form

複数のフォーム要素に一度に入力。

```typescript
mcp__chrome-devtools__fill_form({
  elements: [
    { uid: "2_10", value: "John Doe" },
    { uid: "2_11", value: "john@example.com" },
    { uid: "2_12", value: "password123" }
  ]
})
```

### drag

要素を別の要素にドラッグ&ドロップ。

```typescript
mcp__chrome-devtools__drag({
  from_uid: "3_5",
  to_uid: "3_8"
})
```

### upload_file

ファイル入力要素にファイルをアップロード。

```typescript
mcp__chrome-devtools__upload_file({
  uid: "4_2",
  filePath: "/path/to/file.pdf"
})
```

### handle_dialog

ブラウザダイアログ（alert, confirm, prompt）を処理。

```typescript
mcp__chrome-devtools__handle_dialog({
  action: "accept",  // または "dismiss"
  promptText: "入力テキスト"  // promptの場合のみ
})
```

### wait_for

指定したテキストがページに表示されるまで待機。

```typescript
mcp__chrome-devtools__wait_for({
  text: "読み込み完了",
  timeout: 5000
})
```

**用途：** 動的コンテンツの読み込み待ち

---

## デバッグ

### list_console_messages

コンソールメッセージの一覧を取得。

```typescript
mcp__chrome-devtools__list_console_messages({
  types: ["error", "warn"],  // オプション、フィルター
  pageSize: 50,              // オプション、ページサイズ
  pageIdx: 0,                // オプション、ページ番号
  includePreservedMessages: false  // 過去3ナビゲーション分を含む
})
```

**メッセージタイプ：**
- log, debug, info, error, warn
- dir, dirxml, table, trace
- clear, startGroup, endGroup
- assert, profile, count, timeEnd

### get_console_message

特定のコンソールメッセージの詳細を取得。

```typescript
mcp__chrome-devtools__get_console_message({
  msgid: 123
})
```

### list_network_requests

ネットワークリクエストの一覧を取得。

```typescript
mcp__chrome-devtools__list_network_requests({
  resourceTypes: ["xhr", "fetch", "document"],  // オプション
  pageSize: 100,
  pageIdx: 0,
  includePreservedRequests: false
})
```

**リソースタイプ：**
- document, stylesheet, image, media, font
- script, xhr, fetch, websocket
- prefetch, eventsource, manifest
- ping, cspviolationreport, preflight, other

### get_network_request

特定のネットワークリクエストの詳細を取得。

```typescript
mcp__chrome-devtools__get_network_request({
  reqid: 456
})
```

### evaluate_script

JavaScriptを実行して結果を取得。

```typescript
// 引数なし
mcp__chrome-devtools__evaluate_script({
  function: "() => { return document.title }"
})

// 要素を引数として渡す
mcp__chrome-devtools__evaluate_script({
  function: "(el) => { return el.innerText }",
  args: [{ uid: "1_7" }]
})

// 非同期関数
mcp__chrome-devtools__evaluate_script({
  function: "async () => { return await fetch('/api/data').then(r => r.json()) }"
})
```

**注意：** 返り値はJSON.stringify可能な値のみ

---

## パフォーマンス

### performance_start_trace

パフォーマンストレースの記録を開始。

```typescript
mcp__chrome-devtools__performance_start_trace({
  reload: true,     // ページをリロードするか
  autoStop: true    // 自動停止するか
})
```

### performance_stop_trace

トレースの記録を停止してCore Web Vitalsを取得。

```typescript
mcp__chrome-devtools__performance_stop_trace()
```

**返される指標：**
- LCP (Largest Contentful Paint)
- FCP (First Contentful Paint)
- CLS (Cumulative Layout Shift)
- その他のPerformance Insights

### performance_analyze_insight

特定のパフォーマンスインサイトの詳細を取得。

```typescript
mcp__chrome-devtools__performance_analyze_insight({
  insightName: "LCPBreakdown"
})
```

**利用可能なインサイト：**
- DocumentLatency
- LCPBreakdown
- RenderBlocking
- その他（トレース結果に依存）

---

## エミュレーション

### emulate_network

ネットワーク条件をエミュレート。

```typescript
mcp__chrome-devtools__emulate_network({
  throttlingOption: "Slow 3G"
})
```

**利用可能なオプション：**
- No emulation
- Offline
- Slow 3G
- Fast 3G
- Slow 4G
- Fast 4G

### emulate_cpu

CPU速度をエミュレート。

```typescript
mcp__chrome-devtools__emulate_cpu({
  throttlingRate: 4  // 1-20倍の遅延、1で無効化
})
```

**用途：** 低性能デバイスでのパフォーマンステスト

---

## エラーハンドリング

すべてのツールは失敗時にエラーメッセージを返します。

**一般的なエラー：**
- タイムアウト：指定時間内に完了しなかった
- 要素が見つからない：UIDが無効または要素が削除された
- ナビゲーション失敗：URLが無効またはサーバーエラー
- 権限エラー：ファイルアクセスなどの権限不足

**エラー対処法：**
1. タイムアウトを延長
2. スナップショットで要素の存在を確認
3. wait_forで要素の出現を待つ
4. エラーメッセージから原因を特定

---

## パフォーマンス最適化

### スナップショット vs スクリーンショット

- **スナップショット**：テキストベース、高速、DOM構造の把握に最適
- **スクリーンショット**：画像、遅い、ビジュアル確認が必要な場合のみ使用

### ページの再利用

不要なページクローズ/作成を避け、既存ページを再利用する。

```typescript
// 悪い例：毎回新しいページを作成
await new_page({ url: "http://localhost:5173/page1" })
await close_page({ pageIdx: 0 })
await new_page({ url: "http://localhost:5173/page2" })

// 良い例：navigate_pageで遷移
await new_page({ url: "http://localhost:5173/page1" })
await navigate_page({ url: "http://localhost:5173/page2" })
```

### バッチ操作

可能な限り操作をまとめる。

```typescript
// 良い例：fill_formで一度に入力
await fill_form({
  elements: [
    { uid: "1", value: "name" },
    { uid: "2", value: "email" }
  ]
})
```
