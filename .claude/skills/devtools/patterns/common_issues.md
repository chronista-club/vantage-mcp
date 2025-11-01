# よくある問題と解決方法

Chrome DevTools MCP使用時に遭遇する一般的な問題とその解決策を纏めます。

## 目次

1. [ページナビゲーション関連](#ページナビゲーション関連)
2. [要素の特定と操作](#要素の特定と操作)
3. [タイムアウトとタイミング](#タイムアウトとタイミング)
4. [スナップショット関連](#スナップショット関連)
5. [ネットワークとAPI](#ネットワークとapi)
6. [パフォーマンス問題](#パフォーマンス問題)
7. [エミュレーション関連](#エミュレーション関連)

---

## ページナビゲーション関連

### 問題: ページ遷移後に要素が見つからない

**症状:**
```typescript
await mcp__chrome-devtools__navigate_page({ url: "http://localhost:5173" })
await mcp__chrome-devtools__click({ uid: "nav_button" })
// Error: Element not found
```

**原因:**
- SPAのクライアントサイドルーティングでは即座にDOM更新されない
- 非同期コンポーネントのローディング中

**解決策:**
```typescript
// 1. wait_forで特定のテキストを待つ
await mcp__chrome-devtools__navigate_page({ url: "http://localhost:5173" })
await mcp__chrome-devtools__click({ uid: "nav_button" })
await mcp__chrome-devtools__wait_for({ 
  text: "Dashboard",
  timeout: 5000 
})

// 2. スナップショットを取り直してUIDを再取得
const snapshot = await mcp__chrome-devtools__take_snapshot()
// 新しいスナップショットから要素を探す
```

### 問題: navigate_pageがタイムアウトする

**症状:**
```
Error: Navigation timeout of 30000ms exceeded
```

**原因:**
- サーバーが起動していない
- ネットワークリクエストがハング
- JavaScriptエラーでページロードが完了しない

**解決策:**
```typescript
// 1. 開発サーバーの確認
await mcp__acp__Bash({
  command: "lsof -i :5173",
  description: "開発サーバーが起動しているか確認"
})

// 2. タイムアウトを延長
await mcp__chrome-devtools__navigate_page({ 
  url: "http://localhost:5173",
  timeout: 60000  // 60秒に延長
})

// 3. コンソールエラーを確認
const messages = await mcp__chrome-devtools__list_console_messages({
  types: ["error"]
})
```

---

## 要素の特定と操作

### 問題: UIDが複数マッチしてエラー

**症状:**
```
Error: Multiple elements found with UID "button_123"
```

**原因:**
- スナップショットに同じラベルの要素が複数存在
- UIDの生成アルゴリズムの衝突

**解決策:**
```typescript
// 1. verboseスナップショットで階層情報を取得
const snapshot = await mcp__chrome-devtools__take_snapshot({ 
  verbose: true 
})

// 2. より具体的な親要素を特定してから操作
await mcp__chrome-devtools__click({ uid: "form_123" })  // 親要素を開く
await mcp__chrome-devtools__click({ uid: "submit_button" })  // その中のボタン

// 3. スクリプトで直接操作
await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    const buttons = document.querySelectorAll('button');
    const target = Array.from(buttons).find(b => 
      b.textContent.includes('送信')
    );
    target?.click();
  }`
})
```

### 問題: fill_formで入力できない

**症状:**
```typescript
await mcp__chrome-devtools__fill({ uid: "input_123", value: "test" })
// No error but value not set
```

**原因:**
- Vue/Reactのv-modelやcontrolled componentが入力を上書き
- JavaScriptバリデーションが値をクリア
- 要素が読み取り専用

**解決策:**
```typescript
// 1. evaluate_scriptで直接値をセット
await mcp__chrome-devtools__evaluate_script({
  function: `(el) => {
    el.value = 'test value';
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }`,
  args: [{ uid: "input_123" }]
})

// 2. クリックしてからタイピング
await mcp__chrome-devtools__click({ uid: "input_123" })
await new Promise(resolve => setTimeout(resolve, 100))
await mcp__chrome-devtools__fill({ uid: "input_123", value: "test" })

// 3. 属性を確認
const result = await mcp__chrome-devtools__evaluate_script({
  function: `(el) => ({
    readonly: el.readOnly,
    disabled: el.disabled,
    type: el.type
  })`,
  args: [{ uid: "input_123" }]
})
```

---

## タイムアウトとタイミング

### 問題: wait_forが期待したテキストで反応しない

**症状:**
```typescript
await mcp__chrome-devtools__wait_for({ text: "成功しました" })
// Timeout error
```

**原因:**
- テキストの大文字小文字が一致しない
- 空白文字や改行の違い
- 動的にレンダリングされる前にタイムアウト

**解決策:**
```typescript
// 1. 部分一致で試す
await mcp__chrome-devtools__wait_for({ text: "成功" })

// 2. スナップショットで実際のテキストを確認
const snapshot = await mcp__chrome-devtools__take_snapshot()
console.log(snapshot)  // 実際に表示されているテキストを確認

// 3. evaluate_scriptでポーリング
await mcp__chrome-devtools__evaluate_script({
  function: `async () => {
    for (let i = 0; i < 50; i++) {
      if (document.body.textContent.includes('成功')) {
        return true;
      }
      await new Promise(r => setTimeout(r, 100));
    }
    throw new Error('Timeout waiting for text');
  }`
})
```

### 問題: クリック後の遷移が遅い

**症状:**
```typescript
await mcp__chrome-devtools__click({ uid: "submit" })
await mcp__chrome-devtools__take_snapshot()
// まだローディング画面が表示されている
```

**解決策:**
```typescript
// 1. ローディング要素の消失を待つ
await mcp__chrome-devtools__click({ uid: "submit" })
await mcp__chrome-devtools__evaluate_script({
  function: `async () => {
    while (document.querySelector('.loading-spinner')) {
      await new Promise(r => setTimeout(r, 100));
    }
  }`
})

// 2. 特定の要素の出現を待つ
await mcp__chrome-devtools__wait_for({ text: "完了" })

// 3. ネットワークアイドルを待つ（カスタム実装）
await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    return new Promise(resolve => {
      let timeout;
      const observer = new PerformanceObserver(() => {
        clearTimeout(timeout);
        timeout = setTimeout(resolve, 500);
      });
      observer.observe({ entryTypes: ['resource'] });
    });
  }`
})
```

---

## スナップショット関連

### 問題: スナップショットが大きすぎる

**症状:**
```
Response exceeds maximum size
```

**原因:**
- 大量のリスト項目やテーブル行
- verbose: trueで詳細情報が多すぎる

**解決策:**
```typescript
// 1. verbose: falseで取得
const snapshot = await mcp__chrome-devtools__take_snapshot({ 
  verbose: false 
})

// 2. 特定の要素のみスクリーンショット
await mcp__chrome-devtools__take_screenshot({ 
  uid: "main_content",
  format: "png"
})

// 3. evaluate_scriptで必要な部分のみ抽出
const summary = await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    const items = document.querySelectorAll('.list-item');
    return {
      count: items.length,
      first10: Array.from(items)
        .slice(0, 10)
        .map(el => el.textContent.trim())
    };
  }`
})
```

### 問題: スナップショットに期待した要素が含まれない

**原因:**
- アクセシビリティツリーに含まれない要素（display:none、visibility:hiddenなど）
- ARIA属性が不適切

**解決策:**
```typescript
// 1. スクリーンショットで視覚的に確認
await mcp__chrome-devtools__take_screenshot({ 
  fullPage: false,
  format: "png"
})

// 2. evaluate_scriptで直接DOM確認
const elements = await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    return Array.from(document.querySelectorAll('*'))
      .filter(el => el.offsetParent !== null)
      .map(el => ({
        tag: el.tagName,
        classes: el.className,
        text: el.textContent.substring(0, 50)
      }))
      .slice(0, 20);
  }`
})
```

---

## ネットワークとAPI

### 問題: APIリクエストエラーを捕捉できない

**症状:**
```typescript
const requests = await mcp__chrome-devtools__list_network_requests()
// 失敗したリクエストが見つからない
```

**原因:**
- リクエストが完了する前にlist_network_requestsを呼び出した
- フィルタリング条件が適切でない

**解決策:**
```typescript
// 1. 操作後に待機時間を設ける
await mcp__chrome-devtools__click({ uid: "fetch_data_button" })
await new Promise(resolve => setTimeout(resolve, 1000))

const requests = await mcp__chrome-devtools__list_network_requests({
  resourceTypes: ["xhr", "fetch"]
})

// 2. 特定のURLパターンで詳細取得
for (const req of requests) {
  if (req.url.includes('/api/')) {
    const details = await mcp__chrome-devtools__get_network_request({
      reqid: req.id
    })
    if (details.statusCode >= 400) {
      console.log('Error request:', details)
    }
  }
}

// 3. コンソールエラーも併用
const consoleErrors = await mcp__chrome-devtools__list_console_messages({
  types: ["error"]
})
```

### 問題: CORSエラーでリクエストが失敗

**症状:**
```
Access to fetch at 'http://api.example.com' from origin 'http://localhost:5173' 
has been blocked by CORS policy
```

**解決策:**
```typescript
// Chrome DevToolsでは解決不可、開発サーバー側で対処

// 1. Vite設定でプロキシ設定を追加（vite.config.ts）
// proxy: {
//   '/api': {
//     target: 'http://api.example.com',
//     changeOrigin: true
//   }
// }

// 2. APIサーバーのCORS設定確認

// 3. テスト用にemulate_networkでオフライン状態を確認
await mcp__chrome-devtools__emulate_network({
  throttlingOption: "Offline"
})
// エラーハンドリングをテスト
```

---

## パフォーマンス問題

### 問題: performance_start_traceが完了しない

**症状:**
```typescript
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: false
})
// Never completes
```

**原因:**
- autoStop: falseの場合、手動でstop_traceを呼ぶ必要がある
- ページロードが完了しない

**解決策:**
```typescript
// 1. autoStopを使用
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true  // 自動停止
})

// 2. 手動停止のワークフロー
await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: false
})

// 操作を実行
await mcp__chrome-devtools__click({ uid: "heavy_operation" })
await new Promise(resolve => setTimeout(resolve, 5000))

// トレース停止
const trace = await mcp__chrome-devtools__performance_stop_trace()

// 3. タイムアウト付きで実行
const tracePromise = mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})

const timeoutPromise = new Promise((_, reject) => 
  setTimeout(() => reject(new Error('Trace timeout')), 30000)
)

await Promise.race([tracePromise, timeoutPromise])
```

### 問題: Core Web Vitalsのスコアが悪い

**症状:**
```
LCP: 5.2s (poor)
CLS: 0.35 (needs improvement)
```

**解決策:**
```typescript
// 1. Insightの詳細分析
const trace = await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})

// LCP分析
const lcpInsight = await mcp__chrome-devtools__performance_analyze_insight({
  insightName: "LCPBreakdown"
})

// 2. ネットワークリクエストの確認
const requests = await mcp__chrome-devtools__list_network_requests()
const largeResources = requests.filter(r => r.size > 100000)

// 3. スクリーンショットで視覚確認
await mcp__chrome-devtools__take_screenshot({ 
  fullPage: true,
  format: "png"
})

// 4. 改善後に再測定
// ... コード修正 ...
const newTrace = await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})
```

---

## エミュレーション関連

### 問題: レスポンシブデザインのテストが正しく動作しない

**症状:**
```typescript
await mcp__chrome-devtools__resize_page({ width: 375, height: 667 })
// ブレークポイントが適用されない
```

**原因:**
- CSSメディアクエリがデバイス幅ではなくビューポート幅を参照
- JavaScriptでuserAgentをチェックしている

**解決策:**
```typescript
// 1. モバイルviewportメタタグの確認
const viewport = await mcp__chrome-devtools__evaluate_script({
  function: `() => {
    const meta = document.querySelector('meta[name="viewport"]');
    return meta?.content;
  }`
})

// 2. リサイズ後にページリロード
await mcp__chrome-devtools__resize_page({ width: 375, height: 667 })
await mcp__chrome-devtools__navigate_page({ 
  url: "http://localhost:5173",
  timeout: 10000
})

// 3. スクリーンショットで確認
await mcp__chrome-devtools__take_screenshot({ 
  fullPage: true,
  format: "png",
  filePath: "/tmp/mobile-view.png"
})
```

### 問題: CPU/ネットワークスロットリングの影響が見えない

**原因:**
- ローカル開発環境では効果が限定的
- キャッシュが有効

**解決策:**
```typescript
// 1. キャッシュをクリア（navigate時に実施）
await mcp__chrome-devtools__navigate_page({ url: "http://localhost:5173" })

// 2. 厳しいスロットリング条件
await mcp__chrome-devtools__emulate_network({
  throttlingOption: "Slow 3G"
})

await mcp__chrome-devtools__emulate_cpu({
  throttlingRate: 6  // 6倍遅く
})

// 3. パフォーマンストレースで測定
const trace = await mcp__chrome-devtools__performance_start_trace({
  reload: true,
  autoStop: true
})

// スロットリング解除
await mcp__chrome-devtools__emulate_network({
  throttlingOption: "No emulation"
})
await mcp__chrome-devtools__emulate_cpu({
  throttlingRate: 1
})
```

---

## デバッグのベストプラクティス

### 段階的アプローチ

```typescript
async function debugPageIssue(url: string) {
  // 1. 基本情報収集
  await mcp__chrome-devtools__navigate_page({ url })
  
  const [snapshot, consoleMessages, networkRequests] = await Promise.all([
    mcp__chrome-devtools__take_snapshot({ verbose: false }),
    mcp__chrome-devtools__list_console_messages(),
    mcp__chrome-devtools__list_network_requests()
  ])
  
  // 2. エラー確認
  const errors = consoleMessages.filter(m => m.type === 'error')
  if (errors.length > 0) {
    console.log('Console errors found:', errors.length)
    for (const error of errors) {
      const details = await mcp__chrome-devtools__get_console_message({
        msgid: error.id
      })
      console.log(details)
    }
  }
  
  // 3. ネットワークエラー確認
  const failedRequests = networkRequests.filter(r => 
    r.statusCode >= 400 || r.failed
  )
  
  // 4. スクリーンショット取得
  if (errors.length > 0 || failedRequests.length > 0) {
    await mcp__chrome-devtools__take_screenshot({
      fullPage: true,
      format: "png",
      filePath: `/tmp/debug-${Date.now()}.png`
    })
  }
  
  return {
    errors: errors.length,
    failedRequests: failedRequests.length,
    snapshot
  }
}
```

### ロギングとドキュメント化

```typescript
// 問題の再現手順を記録
const testLog = []

async function loggedAction(description: string, action: () => Promise<any>) {
  testLog.push({ timestamp: new Date(), action: description })
  try {
    const result = await action()
    testLog.push({ timestamp: new Date(), result: 'success' })
    return result
  } catch (error) {
    testLog.push({ 
      timestamp: new Date(), 
      result: 'error', 
      error: error.message 
    })
    throw error
  }
}

// 使用例
await loggedAction('Navigate to dashboard', () =>
  mcp__chrome-devtools__navigate_page({ url: 'http://localhost:5173' })
)

await loggedAction('Click create button', () =>
  mcp__chrome-devtools__click({ uid: 'create_btn' })
)

// エラー時にログを保存
if (testLog.some(entry => entry.result === 'error')) {
  await mcp__acp__Write({
    file_path: `/tmp/test-log-${Date.now()}.json`,
    content: JSON.stringify(testLog, null, 2)
  })
}
```

---

これらのパターンを参考に、効率的なデバッグを実施してください。
