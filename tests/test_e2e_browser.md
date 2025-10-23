# Vantage Browser E2Eテスト（Chrome MCP版）

このテストは、Chrome DevTools MCPサーバーを使用してVantageのWebダッシュボードの動作を検証します。

## 前提条件

- Claude CodeでChrome DevTools MCPサーバーが利用可能
- Vantageがビルド済み

## テスト実行手順

### 1. Vantageサーバーを起動

```bash
cargo run --bin vantage -- --web --web-port 12700
```

サーバーが起動したら、次のメッセージが表示されます：
```
Web dashboard started on http://127.0.0.1:12700
```

### 2. ブラウザページを開く

MCPツール`new_page`を使用してダッシュボードを開きます：

```
url: http://localhost:12700
```

### 3. ダッシュボードの表示を確認

MCPツール`take_snapshot`を使用してページの内容を確認：

**期待される内容**：
- ページタイトルに "Vantage" が含まれる
- `id="app"` の要素が存在する
- Vue.jsアプリケーションが読み込まれている

### 4. プロセス一覧の表示確認

スナップショットから以下を確認：
- プロセス一覧テーブルまたはリストが存在
- 統計情報（total, running, stopped等）が表示される
- "Create Process" 等のアクションボタンがある

### 5. プロセスを作成（API経由）

別のターミナルで以下を実行：

```bash
curl -X POST http://localhost:12700/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "browser-test-echo",
    "command": "echo",
    "args": ["Hello from E2E test"],
    "env": {},
    "cwd": null,
    "auto_start_on_restore": false
  }'
```

### 6. ブラウザをリロードして確認

MCPツール`navigate_page`を使用してリロード：

```
url: http://localhost:12700
```

再度`take_snapshot`でプロセス一覧を確認：

**期待される内容**：
- プロセス一覧に "browser-test-echo" が表示される
- 状態が "Not Started" または "Stopped" と表示される

### 7. プロセスの起動

プロセスの起動ボタンを探してクリック：

1. `take_snapshot`で起動ボタンのUIDを確認
2. MCPツール`click`でボタンをクリック：
   ```
   uid: <起動ボタンのUID>
   ```

### 8. プロセス状態の確認

数秒待機後、再度`take_snapshot`で確認：

**期待される内容**：
- プロセス状態が "Running" または "Stopped" に変わっている
- ログが表示される（"Hello from E2E test" が含まれる）

### 9. APIエンドポイントの直接アクセス

MCPツール`navigate_page`で：

```
url: http://localhost:12700/api/status
```

`take_snapshot`で確認：

**期待される内容**：
```json
{
  "status": "running",
  "version": "0.2.2",
  "uptime_seconds": ...,
  "process_count": ...
}
```

### 10. クリーンアップ

プロセスを削除：

```bash
curl -X DELETE http://localhost:12700/api/processes/browser-test-echo
```

サーバーを停止：
```bash
# Ctrl+C でサーバーを停止
```

## テスト項目チェックリスト

- [ ] ダッシュボードが正常に読み込まれる
- [ ] プロセス一覧が表示される
- [ ] 統計情報が表示される
- [ ] APIで作成したプロセスがブラウザに表示される
- [ ] プロセスの起動ボタンが機能する
- [ ] プロセスのログが表示される
- [ ] APIエンドポイントが正常にレスポンスを返す
- [ ] レイアウトが崩れていない

## トラブルシューティング

### ページが読み込まれない

- サーバーが正常に起動しているか確認
- ポート12700が他のプロセスに使用されていないか確認
- ブラウザのコンソールでエラーがないか確認

### プロセスが表示されない

- APIリクエストが成功したか確認（レスポンスコード201）
- ブラウザをリロードしてみる
- `GET /api/processes` でプロセスが存在するか確認

### ボタンがクリックできない

- `take_snapshot`で正しいUIDを取得しているか確認
- 要素が表示されている（visible）か確認
- JavaScriptのエラーがないか確認

## 自動化の可能性

将来的には、これらの手順をスクリプト化して自動実行することも検討できます：

1. Bashスクリプトでサーバー起動とAPI呼び出し
2. MCPツール呼び出しをJSON形式で記述
3. 期待される結果をアサーション

例：
```bash
#!/bin/bash
# start_server.sh
cargo run --bin vantage -- --web --web-port 12700 &
SERVER_PID=$!
sleep 3

# Claude CodeでMCPツールを実行...

kill $SERVER_PID
```
