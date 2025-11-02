# デバッグパターン集

Chrome DevToolsを使用した効果的なデバッグパターン。

## 基本的なデバッグフロー

### パターン1: エラー優先調査

最も効率的なアプローチは、エラーメッセージから調査を開始すること。

```typescript
// 1. コンソールエラーを確認
const messages = await mcp__chrome-devtools__list_console_messages({
  types: ["error", "warn"]
})

// 2. エラーがあれば詳細を取得
if (messages.length > 0) {
  const detail = await mcp__chrome-devtools__get_console_message({
    msgid: messages[0].msgid
  })
}

// 3. ネットワークエラーを確認
const requests = await mcp__chrome-devtools__list_network_requests({
  resourceTypes: ["xhr", "fetch"]
})

// 4. 失敗したリクエストを特定
const failedRequests = requests.filter(r => r.status >= 400)
```

**使用場面：**
- 新機能追加後の動作確認
- バグレポートの調査
- リグレッションテスト

---

## WebUIエラーの調査

### パターン2: ページが正しく表示されない

```typescript
// 1. ページにアクセス
await mcp__chrome-devtools__navigate_page({
  url: "http://localhost:5173/dashboard"
})

// 2. スナップショットで構造確認
const snapshot = await mcp__chrome-devtools__take_snapshot()
// → 期待する要素が存在するか確認

// 3. コンソールエラー確認
const errors = await mcp__chrome-devtools__list_console_messages({
  types: ["error"]
})

// 4. ネットワークリクエスト確認
const requests = await mcp__chrome-devtools__list_network_requests()
// → APIリクエストが失敗していないか確認

// 5. 必要に応じてスクリーンショット
await mcp__chrome-devtools__take_screenshot({
  fullPage: true,
  filePath: "./debug/dashboard-error.png"
})
```

### パターン3: 動的コンテンツの読み込み問題

```typescript
// 1. ページアクセス
await mcp__chrome-devtools__navigate_page({
  url: "http://localhost:5173/processes"
})

// 2. 要素の出現を待つ
try {
  await mcp__chrome-devtools__wait_for({
    text: "プロセス一覧",
    timeout: 5000
  })
  // → 正常に表示された
} catch (error) {
  // → タイムアウト：読み込み失敗

  // 3. ネットワークリクエストを確認
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["xhr", "fetch"]
  })
  
  // 4. 遅いまたは失敗したリクエストを特定
  const slowRequests = requests.filter(r => r.duration > 3000)
  const failedRequests = requests.filter(r => r.status >= 400)
}
```

### パターン4: JavaScript実行時エラー

```typescript
// 1. エラーメッセージを収集
const messages = await mcp__chrome-devtools__list_console_messages({
  types: ["error"]
})

// 2. 各エラーの詳細を確認
for (const msg of messages) {
  const detail = await mcp__chrome-devtools__get_console_message({
    msgid: msg.msgid
  })
  
  // スタックトレースやソースファイルを確認
  // → エラー発生箇所を特定
}

// 3. 状態を確認するためJavaScript実行
const state = await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    return {
      storeState: window.__VUE_DEVTOOLS_GLOBAL_HOOK__,
      router: window.router,
      errors: window.errors
    }
  }`
})
```

---

## APIエラーのデバッグ

### パターン5: 失敗したAPIリクエストの調査

```typescript
// 1. ネットワークリクエストを取得
const requests = await mcp__chrome-devtools__list_network_requests({
  resourceTypes: ["xhr", "fetch"]
})

// 2. 失敗したリクエストをフィルター
const failed = requests.filter(r => r.status >= 400 || r.status === 0)

// 3. 各失敗リクエストの詳細を確認
for (const req of failed) {
  const detail = await mcp__chrome-devtools__get_network_request({
    reqid: req.reqid
  })
  
  // レスポンスボディ、ヘッダー、タイミングを確認
  console.log({
    url: detail.url,
    status: detail.status,
    responseBody: detail.responseBody,
    requestHeaders: detail.requestHeaders,
    timing: detail.timing
  })
}
```

### パターン6: APIレスポンスタイムの問題

```typescript
// 1. パフォーマンストレース開始
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})

// 2. トレース停止と結果取得
const trace = await mcp__chrome-devtools__performance_stop_trace()

// 3. 遅いネットワークリクエストを確認
const requests = await mcp__chrome-devtools__list_network_requests()
const slowRequests = requests
  .filter(r => r.duration > 1000)
  .sort((a, b) => b.duration - a.duration)

// 4. 詳細を調査
for (const req of slowRequests.slice(0, 5)) {
  const detail = await mcp__chrome-devtools__get_network_request({
    reqid: req.reqid
  })
  // タイミング詳細を確認
}
```

---

## フォーム関連のデバッグ

### パターン7: フォーム送信の失敗

```typescript
// 1. フォームページにアクセス
await mcp__chrome-devtools__navigate_page({
  url: "http://localhost:5173/template/create"
})

// 2. スナップショットで要素確認
const snapshot = await mcp__chrome-devtools__take_snapshot()

// 3. フォーム入力
await mcp__chrome-devtools__fill_form({
  elements: [
    { uid: "form_name", value: "Test Template" },
    { uid: "form_command", value: "echo 'test'" }
  ]
})

// 4. 送信ボタンをクリック
await mcp__chrome-devtools__click({ uid: "submit_button" })

// 5. 結果を確認
try {
  await mcp__chrome-devtools__wait_for({
    text: "作成しました",
    timeout: 3000
  })
  // → 成功
} catch {
  // → 失敗：エラーメッセージを確認
  const errors = await mcp__chrome-devtools__list_console_messages({
    types: ["error"]
  })
  
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["xhr", "fetch"]
  })
}
```

---

## レスポンシブデザインのデバッグ

### パターン8: モバイルビューの問題

```typescript
// 1. ページサイズを変更
await mcp__chrome-devtools__resize_page({
  width: 375,
  height: 812
})

// 2. ページをリロード
await mcp__chrome-devtools__navigate_page({
  url: "http://localhost:5173"
})

// 3. レイアウトを確認
const snapshot = await mcp__chrome-devtools__take_snapshot()

// 4. スクリーンショットで視覚確認
await mcp__chrome-devtools__take_screenshot({
  fullPage: true,
  filePath: "./debug/mobile-view.png"
})

// 5. 様々な画面サイズでテスト
const sizes = [
  { width: 375, height: 812, name: "iPhone" },
  { width: 768, height: 1024, name: "iPad" },
  { width: 1920, height: 1080, name: "Desktop" }
]

for (const size of sizes) {
  await mcp__chrome-devtools__resize_page(size)
  await mcp__chrome-devtools__take_screenshot({
    fullPage: true,
    filePath: `./debug/${size.name}.png`
  })
}
```

---

## パフォーマンスデバッグ

### パターン9: Core Web Vitalsの測定

```typescript
// 1. トレース開始（ページリロード付き）
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})

// 2. トレース停止と結果取得
const trace = await mcp__chrome-devtools__performance_stop_trace()

// 3. 主要指標を確認
console.log({
  LCP: trace.insights.LCP,
  FCP: trace.insights.FCP,
  CLS: trace.insights.CLS
})

// 4. 問題があればインサイトを詳しく確認
if (trace.insights.LCP.value > 2500) {
  const lcpDetail = await mcp__chrome-devtools__performance_analyze_insight({
    insightName: "LCPBreakdown"
  })
}
```

### パターン10: 遅いレンダリングの調査

```typescript
// 1. CPU throttlingを有効化
await mcp__chrome-devtools__emulate_cpu({
  throttlingRate: 4  // 4倍遅く
})

// 2. パフォーマンストレース
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: false
})

// 3. 操作を実行（例：リストのソート）
await mcp__chrome-devtools__click({ uid: "sort_button" })

// 4. トレース停止
const trace = await mcp__chrome-devtools__performance_stop_trace()

// 5. throttling解除
await mcp__chrome-devtools__emulate_cpu({
  throttlingRate: 1
})
```

---

## デバッグセッションのテンプレート

### 完全なデバッグフロー

```typescript
async function debugPage(url: string) {
  // 1. ページアクセス
  await mcp__chrome-devtools__navigate_page({ url })
  
  // 2. 基本情報収集
  const [snapshot, errors, requests] = await Promise.all([
    mcp__chrome-devtools__take_snapshot(),
    mcp__chrome-devtools__list_console_messages({ types: ["error", "warn"] }),
    mcp__chrome-devtools__list_network_requests()
  ])
  
  // 3. エラー分析
  const hasErrors = errors.length > 0
  const failedRequests = requests.filter(r => r.status >= 400)
  
  if (hasErrors) {
    console.log("=== Console Errors ===")
    for (const err of errors) {
      const detail = await mcp__chrome-devtools__get_console_message({
        msgid: err.msgid
      })
      console.log(detail)
    }
  }
  
  if (failedRequests.length > 0) {
    console.log("=== Failed Requests ===")
    for (const req of failedRequests) {
      const detail = await mcp__chrome-devtools__get_network_request({
        reqid: req.reqid
      })
      console.log(detail)
    }
  }
  
  // 4. スクリーンショット（問題がある場合）
  if (hasErrors || failedRequests.length > 0) {
    await mcp__chrome-devtools__take_screenshot({
      fullPage: true,
      filePath: `./debug/${new Date().toISOString()}.png`
    })
  }
  
  return {
    snapshot,
    errors,
    failedRequests
  }
}
```

---

## トラブルシューティングのチェックリスト

1. **コンソールエラー** - 最初に確認
2. **ネットワークリクエスト** - API通信の問題
3. **DOM構造** - スナップショットで要素の存在確認
4. **タイミング** - wait_forで動的コンテンツの読み込み待ち
5. **スクリーンショット** - ビジュアル確認が必要な場合
6. **JavaScript実行** - 状態やグローバル変数の確認
7. **パフォーマンス** - 遅延の原因特定

## ベストプラクティス

### DO ✅
- エラーメッセージから調査開始
- スナップショットを活用（高速）
- 複数の情報源を組み合わせる
- 再現可能なステップを記録

### DON'T ❌
- スクリーンショットに頼りすぎない
- タイムアウトを短くしすぎない
- エラーログを無視しない
- 一度に多くのページを開きすぎない
