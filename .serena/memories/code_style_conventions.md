# Ichimi Server コーディング規約

## Rustコードスタイル

### 命名規則
- **構造体/Enum**: PascalCase（例: `ProcessManager`, `ProcessState`）
- **関数/メソッド**: snake_case（例: `create_process`, `get_status`）
- **定数**: SCREAMING_SNAKE_CASE（例: `DEFAULT_BUFFER_SIZE`）
- **モジュール**: snake_case（例: `process_manager`）

### 型とエラーハンドリング
- `Result<T, String>` を広く使用（簡易エラー型）
- 本番コードでは `anyhow::Result` を使用
- `Arc<RwLock<T>>` で共有状態を管理
- `async/await` を積極的に使用

### ドキュメント
- モジュールレベルに `///` コメント
- 公開APIには必ずドキュメント
- 日本語コメントOK（内部実装の説明）

### コード構成
```rust
// 1. use文（外部クレート → 標準ライブラリ → 内部モジュール）
use anyhow::Result;
use std::sync::Arc;
use crate::process::ProcessManager;

// 2. 型定義
pub struct Server {
    // ...
}

// 3. impl ブロック
impl Server {
    pub async fn new() -> Result<Self> {
        // ...
    }
}

// 4. プライベート関数
fn helper_function() {
    // ...
}
```

### テスト
- ユニットテストは同じファイルの `#[cfg(test)]` モジュール内
- 統合テストは `tests/` ディレクトリ
- `cargo test` で全テスト実行

### フォーマットとLint
- `cargo fmt` でフォーマット（コミット前に必須）
- `cargo clippy` でLintチェック
- 警告は可能な限り解消

## MCPツール定義規約
```rust
#[tool(description = "ツールの説明")]
async fn tool_name(
    &self,
    Parameters(Request { field }): Parameters<Request>,
) -> Result<CallToolResult, McpError> {
    // 実装
}
```

## Git コミットメッセージ
- 日本語または英語
- prefix: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`
- 例: `feat: SurrealDB永続化を実装`