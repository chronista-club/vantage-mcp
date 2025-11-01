# テストパターン集

Chrome DevToolsを使用したWebアプリケーションのテストパターン。

## E2Eテストの基本パターン

### パターン1: ログイン→操作→検証

```typescript
async function testUserFlow() {
  // 1. ログインページにアクセス
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/login"
  })
  
  // 2. 認証情報を入力
  await mcp__chrome-devtools__fill_form({
    elements: [
      { uid: "username_input", value: "test@example.com" },
      { uid: "password_input", value: "password123" }
    ]
  })
  
  // 3. ログインボタンをクリック
  await mcp__chrome-devtools__click({ uid: "login_button" })
  
  // 4. ダッシュボードへ遷移したことを確認
  await mcp__chrome-devtools__wait_for({
    text: "ダッシュボード",
    timeout: 5000
  })
  
  // 5. スナップショットで状態確認
  const snapshot = await mcp__chrome-devtools__take_snapshot()
  
  // 6. エラーがないことを確認
  const errors = await mcp__chrome-devtools__list_console_messages({
    types: ["error"]
  })
  
  return errors.length === 0
}
```

---

## フォームテスト

### パターン2: フォーム入力と検証

```typescript
async function testFormSubmission() {
  // 1. フォームページにアクセス
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/template/create"
  })
  
  // 2. スナップショットで要素確認
  const snapshot1 = await mcp__chrome-devtools__take_snapshot()
  
  // 3. フォームに入力
  await mcp__chrome-devtools__fill_form({
    elements: [
      { uid: "name_input", value: "Test Template" },
      { uid: "command_input", value: "npm start" },
      { uid: "description_input", value: "テスト用テンプレート" }
    ]
  })
  
  // 4. 送信
  await mcp__chrome-devtools__click({ uid: "submit_button" })
  
  // 5. 成功メッセージを待つ
  await mcp__chrome-devtools__wait_for({
    text: "作成しました",
    timeout: 3000
  })
  
  // 6. APIリクエストが成功したことを確認
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["xhr", "fetch"]
  })
  
  const createRequest = requests.find(r => 
    r.url.includes("/api/templates") && r.method === "POST"
  )
  
  return createRequest && createRequest.status === 200
}
```

### パターン3: バリデーションテスト

```typescript
async function testFormValidation() {
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/template/create"
  })
  
  // 必須フィールドを空のまま送信
  await mcp__chrome-devtools__click({ uid: "submit_button" })
  
  // バリデーションエラーが表示されることを確認
  await mcp__chrome-devtools__wait_for({
    text: "必須項目です",
    timeout: 2000
  })
  
  // フォームが送信されていないことを確認
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["xhr", "fetch"]
  })
  
  const postRequests = requests.filter(r => 
    r.method === "POST" && r.url.includes("/api/templates")
  )
  
  return postRequests.length === 0
}
```

---

## ナビゲーションテスト

### パターン4: SPA(Single Page Application)のルーティングテスト

```typescript
async function testNavigation() {
  const pages = [
    { path: "/dashboard", expectedText: "ダッシュボード" },
    { path: "/processes", expectedText: "プロセス管理" },
    { path: "/templates", expectedText: "テンプレート" },
    { path: "/clipboard", expectedText: "クリップボード" }
  ]
  
  for (const page of pages) {
    // 1. ページに遷移
    await mcp__chrome-devtools__navigate_page({
      url: `http://localhost:5173${page.path}`
    })
    
    // 2. 期待するテキストが表示されるまで待つ
    await mcp__chrome-devtools__wait_for({
      text: page.expectedText,
      timeout: 3000
    })
    
    // 3. JavaScriptエラーがないことを確認
    const errors = await mcp__chrome-devtools__list_console_messages({
      types: ["error"]
    })
    
    if (errors.length > 0) {
      console.error(`Errors on ${page.path}:`, errors)
      return false
    }
  }
  
  return true
}
```

### パターン5: ブラウザ履歴のテスト

```typescript
async function testBrowserHistory() {
  // 1. 複数ページを訪問
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/dashboard"
  })
  
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/processes"
  })
  
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/templates"
  })
  
  // 2. 戻るボタンで前のページへ
  await mcp__chrome-devtools__navigate_page_history({
    navigate: "back"
  })
  
  // 3. プロセスページに戻ったことを確認
  await mcp__chrome-devtools__wait_for({
    text: "プロセス管理",
    timeout: 2000
  })
  
  // 4. さらに戻る
  await mcp__chrome-devtools__navigate_page_history({
    navigate: "back"
  })
  
  // 5. ダッシュボードに戻ったことを確認
  await mcp__chrome-devtools__wait_for({
    text: "ダッシュボード",
    timeout: 2000
  })
  
  // 6. 進むボタンでテンプレートページへ
  await mcp__chrome-devtools__navigate_page_history({
    navigate: "forward"
  })
  
  return true
}
```

---

## APIインテグレーションテスト

### パターン6: CRUD操作のテスト

```typescript
async function testCRUDOperations() {
  // CREATE
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/template/create"
  })
  
  await mcp__chrome-devtools__fill_form({
    elements: [
      { uid: "name", value: "E2E Test Template" },
      { uid: "command", value: "echo test" }
    ]
  })
  
  await mcp__chrome-devtools__click({ uid: "submit" })
  await mcp__chrome-devtools__wait_for({ text: "作成しました" })
  
  // READ - リストに表示されることを確認
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/templates"
  })
  
  await mcp__chrome-devtools__wait_for({ 
    text: "E2E Test Template" 
  })
  
  // UPDATE
  await mcp__chrome-devtools__click({ uid: "edit_button" })
  await mcp__chrome-devtools__fill({ 
    uid: "name", 
    value: "Updated Template" 
  })
  await mcp__chrome-devtools__click({ uid: "save" })
  await mcp__chrome-devtools__wait_for({ text: "更新しました" })
  
  // DELETE
  await mcp__chrome-devtools__click({ uid: "delete_button" })
  await mcp__chrome-devtools__handle_dialog({ action: "accept" })
  await mcp__chrome-devtools__wait_for({ text: "削除しました" })
  
  return true
}
```

### パターン7: リアルタイム更新のテスト

```typescript
async function testRealtimeUpdates() {
  // 1. プロセスページを開く
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/processes"
  })
  
  // 2. 初期状態のスナップショット
  const before = await mcp__chrome-devtools__take_snapshot()
  
  // 3. プロセスを起動
  await mcp__chrome-devtools__click({ uid: "start_process_button" })
  
  // 4. ステータスが「実行中」に変わるまで待つ
  await mcp__chrome-devtools__wait_for({
    text: "実行中",
    timeout: 5000
  })
  
  // 5. 更新後のスナップショット
  const after = await mcp__chrome-devtools__take_snapshot()
  
  // 6. ネットワークリクエストでSSE/WebSocket接続を確認
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["websocket", "eventsource"]
  })
  
  return requests.length > 0
}
```

---

## UIインタラクションテスト

### パターン8: モーダルダイアログのテスト

```typescript
async function testModal() {
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/templates"
  })
  
  // 1. モーダルを開く
  await mcp__chrome-devtools__click({ uid: "create_button" })
  
  // 2. モーダルが表示されたことを確認
  await mcp__chrome-devtools__wait_for({
    text: "テンプレートを作成",
    timeout: 2000
  })
  
  // 3. フォームに入力
  await mcp__chrome-devtools__fill_form({
    elements: [
      { uid: "modal_name", value: "Modal Test" },
      { uid: "modal_command", value: "test" }
    ]
  })
  
  // 4. キャンセルボタンをクリック
  await mcp__chrome-devtools__click({ uid: "modal_cancel" })
  
  // 5. モーダルが閉じたことを確認（テキストが消える）
  try {
    await mcp__chrome-devtools__wait_for({
      text: "テンプレートを作成",
      timeout: 1000
    })
    return false // モーダルがまだ開いている
  } catch {
    return true // 期待通りモーダルが閉じた
  }
}
```

### パターン9: ドラッグ&ドロップのテスト

```typescript
async function testDragAndDrop() {
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173/processes"
  })
  
  // 1. ドラッグ前の状態を記録
  const before = await mcp__chrome-devtools__take_snapshot()
  
  // 2. ドラッグ&ドロップを実行
  await mcp__chrome-devtools__drag({
    from_uid: "process_1",
    to_uid: "reorder_zone"
  })
  
  // 3. 並び順が変わったことを確認
  const after = await mcp__chrome-devtools__take_snapshot()
  
  // 4. APIリクエストが送信されたことを確認
  const requests = await mcp__chrome-devtools__list_network_requests({
    resourceTypes: ["xhr", "fetch"]
  })
  
  const reorderRequest = requests.find(r => 
    r.url.includes("/api/processes/reorder")
  )
  
  return reorderRequest && reorderRequest.status === 200
}
```

---

## レスポンシブデザインテスト

### パターン10: マルチデバイステスト

```typescript
async function testResponsiveDesign() {
  const devices = [
    { name: "Mobile", width: 375, height: 812 },
    { name: "Tablet", width: 768, height: 1024 },
    { name: "Desktop", width: 1920, height: 1080 }
  ]
  
  for (const device of devices) {
    // 1. デバイスサイズに設定
    await mcp__chrome-devtools__resize_page({
      width: device.width,
      height: device.height
    })
    
    // 2. ページをリロード
    await mcp__chrome-devtools__navigate_page({
      url: "http://localhost:5173/dashboard"
    })
    
    // 3. レイアウトが正しく表示されることを確認
    const snapshot = await mcp__chrome-devtools__take_snapshot()
    
    // 4. レスポンシブメニューが適切に表示されるか確認
    if (device.width < 768) {
      // モバイル：ハンバーガーメニュー
      await mcp__chrome-devtools__wait_for({
        text: "メニュー",
        timeout: 2000
      })
    } else {
      // デスクトップ：通常のナビゲーション
      await mcp__chrome-devtools__wait_for({
        text: "ダッシュボード",
        timeout: 2000
      })
    }
    
    // 5. スクリーンショット保存
    await mcp__chrome-devtools__take_screenshot({
      fullPage: true,
      filePath: `./screenshots/${device.name}.png`
    })
  }
  
  return true
}
```

---

## パフォーマンステスト

### パターン11: ページ読み込み速度のテスト

```typescript
async function testPageLoadPerformance() {
  // 1. パフォーマンストレース開始
  await mcp__chrome-devtools__performance_start_trace({
    reload: true,
    autoStop: true
  })
  
  // 2. トレース停止と結果取得
  const trace = await mcp__chrome-devtools__performance_stop_trace()
  
  // 3. Core Web Vitalsを検証
  const passed = {
    LCP: trace.insights.LCP.value < 2500,  // 2.5秒以下
    FCP: trace.insights.FCP.value < 1800,  // 1.8秒以下
    CLS: trace.insights.CLS.value < 0.1    // 0.1以下
  }
  
  // 4. 結果をログ
  console.log({
    LCP: `${trace.insights.LCP.value}ms (${passed.LCP ? "PASS" : "FAIL"})`,
    FCP: `${trace.insights.FCP.value}ms (${passed.FCP ? "PASS" : "FAIL"})`,
    CLS: `${trace.insights.CLS.value} (${passed.CLS ? "PASS" : "FAIL"})`
  })
  
  return Object.values(passed).every(p => p)
}
```

### パターン12: 低速ネットワークでのテスト

```typescript
async function testSlowNetwork() {
  // 1. ネットワークを遅く設定
  await mcp__chrome-devtools__emulate_network({
    throttlingOption: "Slow 3G"
  })
  
  // 2. ページにアクセス
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173"
  })
  
  // 3. ローディング表示が出ることを確認
  await mcp__chrome-devtools__wait_for({
    text: "読み込み中",
    timeout: 1000
  })
  
  // 4. コンテンツが最終的に表示されることを確認
  await mcp__chrome-devtools__wait_for({
    text: "ダッシュボード",
    timeout: 30000
  })
  
  // 5. ネットワーク設定をリセット
  await mcp__chrome-devtools__emulate_network({
    throttlingOption: "No emulation"
  })
  
  return true
}
```

---

## アクセシビリティテスト

### パターン13: キーボードナビゲーションのテスト

```typescript
async function testKeyboardNavigation() {
  await mcp__chrome-devtools__navigate_page({
    url: "http://localhost:5173"
  })
  
  // JavaScriptでキーボードイベントをシミュレート
  await mcp__chrome-devtools__evaluate_script({
    function: `() => {
      const tabEvent = new KeyboardEvent('keydown', { key: 'Tab' })
      document.dispatchEvent(tabEvent)
      
      // フォーカスされた要素を取得
      return {
        tagName: document.activeElement.tagName,
        role: document.activeElement.getAttribute('role'),
        text: document.activeElement.innerText
      }
    }`
  })
  
  // スナップショットで要素のフォーカス状態を確認
  const snapshot = await mcp__chrome-devtools__take_snapshot()
  
  return true
}
```

---

## テストスイートの例

### 完全なテストスイート

```typescript
async function runTestSuite() {
  const results = {
    navigation: false,
    formSubmission: false,
    formValidation: false,
    performance: false,
    responsive: false
  }
  
  try {
    // 1. ナビゲーションテスト
    console.log("Running navigation tests...")
    results.navigation = await testNavigation()
    
    // 2. フォームテスト
    console.log("Running form tests...")
    results.formSubmission = await testFormSubmission()
    results.formValidation = await testFormValidation()
    
    // 3. パフォーマンステスト
    console.log("Running performance tests...")
    results.performance = await testPageLoadPerformance()
    
    // 4. レスポンシブテスト
    console.log("Running responsive tests...")
    results.responsive = await testResponsiveDesign()
    
    // 5. 結果サマリー
    const passed = Object.values(results).filter(r => r).length
    const total = Object.keys(results).length
    
    console.log(`\n=== Test Results ===`)
    console.log(`Passed: ${passed}/${total}`)
    console.log(results)
    
    return passed === total
  } catch (error) {
    console.error("Test suite failed:", error)
    return false
  }
}
```

---

## ベストプラクティス

### テスト設計

1. **独立したテスト**：各テストは他のテストに依存しない
2. **クリーンアップ**：テスト後にデータを削除
3. **適切なタイムアウト**：環境に応じて調整
4. **エラーハンドリング**：失敗時の詳細情報を記録

### テスト実行

1. **並列実行**：独立したテストは並列で実行可能
2. **リトライ**：不安定なテストは複数回実行
3. **スクリーンショット**：失敗時に自動保存
4. **継続的インテグレーション**：CI/CDパイプラインに統合

### デバッグ

1. **詳細なログ**：各ステップの結果を記録
2. **スナップショット保存**：失敗時の状態を保存
3. **ネットワークログ**：APIリクエストを確認
4. **コンソールログ**：JavaScriptエラーを確認
