# CLAUDE.md

このファイルは、このリポジトリのコードを扱う際の Claude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

Ichimi Server は Model Context Protocol (MCP) を介した Claude Code 用のプロセス管理サーバーです。Claude がプロセスの起動、停止、監視、および MCP ツールを通じた出力のキャプチャを可能にします。

## ビルド・開発コマンド

```bash
# ビルドコマンド
cargo build           # デバッグビルド
cargo build --release # リリースビルド（最適化済み）

# テスト
cargo test           # 全テストを実行
cargo test [test_name] # 特定のテストを実行

# コード品質
cargo fmt            # コードをフォーマット
cargo fmt -- --check # ファイルを変更せずにフォーマットをチェック
cargo clippy         # リンターを実行
cargo clippy -- -D warnings # 警告でエラーにする

# サーバーの実行
cargo run --bin ichimi
./target/release/ichimi # リリースビルドを実行
```

## アーキテクチャ

### モジュール構造

コードベースは機能別モジュールに整理されています：

- **`src/lib.rs`**: MCP ツールハンドラーを持つメインサーバー実装。各ツールメソッドは `#[tool]` 属性で装飾され、Claude に公開される MCP ツールにマッピングされます。

- **`src/messages/`**: リクエスト/レスポンスメッセージ構造体
  - `basic.rs`: シンプルなメッセージタイプ（echo、ping）
  - `process.rs`: プロセス管理リクエストタイプ
  
- **`src/process/`**: コアプロセス管理ロジック
  - `manager.rs`: `ProcessManager` - プロセスライフサイクルを処理し、プロセスレジストリを維持
  - `buffer.rs`: `CircularBuffer` - 固定容量でメモリ効率的なログストレージ
  - `types.rs`: ドメインタイプ（`ProcessState`、`ProcessInfo`、`ProcessStatus`）

### 主要な設計パターン

1. **Arc<RwLock> パターン**: `ProcessManager` は管理対象プロセスへのスレッドセーフな並行アクセスのために `Arc<RwLock<HashMap>>` を使用。各プロセスも細かいロック制御のために `Arc<RwLock>` でラップされています。

2. **ステートマシン**: プロセスは状態を遷移します：`NotStarted` → `Running` → `Stopped`/`Failed`。状態遷移はアトミックで、タイムスタンプを含みます。

3. **非同期出力キャプチャ**: プロセス開始時、stdout/stderr を循環バッファにキャプチャする2つの非同期タスクが生成され、長時間実行プロセスによるメモリ枯渇を防ぎます。

4. **ツールルーター**: `#[tool_router]` マクロが MCP ツールルーティングを生成。ツールは `CallToolResult` を返す非同期関数です。

## MCP 統合ポイント

サーバーは以下のツールを Claude に公開します：
- 基本: `echo`、`ping`、`get_status`
- プロセス管理: `create_process`、`start_process`、`stop_process`、`get_process_status`、`get_process_output`、`list_processes`、`remove_process`

各ツールは `lib.rs` の `IchimiServer` impl ブロック内のメソッドに直接マッピングされます。

## プロセスライフサイクル

1. **作成**: プロセス設定を登録（command、args、env、cwd）
2. **起動**: tokio プロセスを生成、PID をキャプチャ、出力ハンドラーを開始
3. **監視**: 状態を追跡、stdout/stderr を循環バッファにキャプチャ
4. **停止**: SIGTERM を送信、猶予期間を待機、必要に応じて強制終了
5. **削除**: レジストリからプロセスをクリーンアップ

## エラーハンドリング

- すべてのプロセス操作は `Result<T, String>` を返す
- エラーは `ErrorCode::INTERNAL_ERROR` を持つ MCP エラーに変換される
- プロセスの失敗は、エラー詳細と共に `ProcessState::Failed` にキャプチャされる

## テストに関する考慮事項

現在、ユニットテストは存在しません。テスト追加時は：
- プロセス操作のために `tokio::process::Command` をモック化
- `ProcessManager` での状態遷移をテスト
- 容量での循環バッファの動作を検証
- 並行アクセスパターンをテスト