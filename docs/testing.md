# Ichimiサーバーのテスト戦略

## テスト概要

Ichimiサーバーには以下のテストが実装されています：

### 1. Web APIテスト（✅ 動作確認済み）

**ファイル**: `crates/ichimi/tests/test_web_api.rs`

**実行方法**:
```bash
cargo test --test test_web_api --features web
```

**テストカバレッジ**:
- ✅ サーバーステータス取得 (`GET /status`)
- ✅ ダッシュボードデータ取得 (`GET /dashboard`)
- ✅ プロセスライフサイクル
  - プロセス作成 (`POST /processes`)
  - プロセス一覧 (`GET /processes`)
  - プロセス起動 (`POST /processes/:id/start`)
  - プロセスログ取得 (`GET /processes/:id/logs`)
  - プロセス削除 (`DELETE /processes/:id`)
- ✅ プロセスフィルタリング（状態、名前パターン）

**実装の特徴**:
- 実際のHTTPサーバーを起動してテスト
- Axum + reqwestによる統合テスト
- 並行実行を避けるため`--test-threads=1`を推奨

**テスト結果**:
```
running 4 tests
test test_dashboard_endpoint ... ok
test test_get_status ... ok
test test_process_filtering ... ok
test test_process_lifecycle_api ... ok

test result: ok. 4 passed; 0 failed
```

### 2. ブラウザE2Eテスト（✅ Chrome MCP DevTools版）

**推奨方法**: Chrome MCP DevToolsを使用した対話的E2Eテスト

**ドキュメント**: `docs/testing-chrome-mcp.md`

**実行方法**:
Claude Codeセッション内でMCPツールを使用
```bash
# 1. サーバー起動
cargo run --bin ichimi -- --web --web-port 12700

# 2. Chrome MCPツールで操作
# - mcp__chrome-devtools__new_page
# - mcp__chrome-devtools__take_snapshot
# - mcp__chrome-devtools__click
# - mcp__chrome-devtools__take_screenshot
```

**テスト項目**:
- ✅ ダッシュボード表示
- ✅ プロセス一覧表示
- ✅ プロセス作成・起動・停止
- ✅ ログ表示
- ✅ 統計情報の更新

**利点**:
- 依存関係不要（MCPサーバー使用）
- 視覚的確認が容易（スクリーンショット）
- デバッグが簡単（スナップショット）
- 保守性が高い

**実行結果**: 全項目成功（2025-10-08実施）

---

### 2-alt. ブラウザテスト（headless_chrome版・非推奨）

**ファイル**: `crates/ichimi/tests/test_browser.rs`

**注意**: Chrome MCP版を推奨。この方式は制約が多い。

**制約事項**:
- headless_chromeに依存（Chromiumが必要）
- テストサーバーでは完全なアプリケーションルーターが必要
- 現状、`create_app`関数が公開されていないため動作しない

### 3. 既存のユニット・統合テスト

**ファイル**:
- `crates/ichimi/tests/test_integration.rs` - プロセス管理の統合テスト
- `crates/ichimi/tests/test_process_update.rs` - プロセス更新のテスト

**実行方法**:
```bash
cargo test
```

## 推奨テスト戦略

### 開発時
```bash
# 全テストを実行
cargo test

# APIテストのみ実行
cargo test --test test_web_api --features web
```

### CI/CD
```bash
# 全テストを実行（ブラウザテストを除く）
cargo test --features web

# フォーマットチェック
cargo fmt -- --check

# Lintチェック
cargo clippy -- -D warnings
```

## テストの拡張

### 新しいAPIエンドポイントを追加した場合

1. `test_web_api.rs`に新しいテストケースを追加
2. エンドポイントの正常系と異常系を両方テスト
3. レスポンス形式を検証

### 例：新しいテンプレート機能のテスト
```rust
#[tokio::test]
async fn test_template_crud() {
    // 1. テンプレート作成
    // 2. テンプレート一覧取得
    // 3. テンプレート更新
    // 4. テンプレート削除
}
```

## トラブルシューティング

### ポート競合エラー
サーバーは自動的に空いているポートを探しますが、並行テストを避けるため：
```bash
cargo test --test test_web_api -- --test-threads=1
```

### タイムアウトエラー
テストのタイムアウトを延長：
```rust
tokio::time::sleep(Duration::from_secs(2)).await;
```

## まとめ

- ✅ **APIテスト**: 本番レベルの品質保証に十分
- 🚧 **ブラウザテスト**: 現時点では制約あり、手動テストを推奨
- 🎯 **今後**: E2Eテストフレームワーク（Playwright等）の導入を検討
