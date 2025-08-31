# CLAUDE.md

このファイルは、このリポジトリのコードを扱う際の Claude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

Ichimi Server は Model Context Protocol (MCP) を介した Claude Code 用のプロセス管理サーバーです。Claude がプロセスの起動、停止、監視、および MCP ツールを通じた出力のキャプチャを可能にします。

### 主な機能
- プロセスのライフサイクル管理（作成、起動、停止、削除）
- リアルタイムログキャプチャ（stdout/stderr）
- SurrealDB インメモリデータベースによる永続化
- .surql ファイルへのエクスポート/インポート機能
- Webダッシュボード（Alpine.js + Tabler UI）
- 自動バックアップ機能

## インストール方法

```bash
# GitHubから特定のバージョンを直接インストール（推奨）
cargo install --git https://github.com/chronista-club/ichimi-server --tag v0.1.0-beta10

# 最新のmainブランチからインストール
cargo install --git https://github.com/chronista-club/ichimi-server

# ローカルでビルドしてインストール
git clone https://github.com/chronista-club/ichimi-server.git
cd ichimi-server
cargo install --path .
```

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

# Webダッシュボード付きで実行
cargo run --bin ichimi -- --web
cargo run --bin ichimi -- --web --web-port 8080  # カスタムポート

# 環境変数を設定して実行
RUST_LOG=debug cargo run
ICHIMI_AUTO_EXPORT_INTERVAL=300 cargo run  # 5分ごとに自動エクスポート
```

## アーキテクチャ

### モジュール構造

コードベースは機能別モジュールに整理されています：

- **`src/lib.rs`**: MCP ツールハンドラーを持つメインサーバー実装。各ツールメソッドは `#[tool]` 属性で装飾され、Claude に公開される MCP ツールにマッピングされます。

- **`src/messages/`**: リクエスト/レスポンスメッセージ構造体
  - `basic.rs`: シンプルなメッセージタイプ（echo、ping）
  - `process.rs`: プロセス管理リクエストタイプ
  - `ci.rs`: CI監視リクエストタイプ
  
- **`src/process/`**: コアプロセス管理ロジック
  - `manager.rs`: `ProcessManager` - プロセスライフサイクルを処理し、プロセスレジストリを維持
  - `buffer.rs`: `CircularBuffer` - 固定容量でメモリ効率的なログストレージ
  - `types.rs`: ドメインタイプ（`ProcessState`、`ProcessInfo`、`ProcessStatus`）

- **`src/persistence.rs`**: SurrealDB インメモリデータベースによる永続化層
  - `PersistenceManager` - プロセスデータの保存/読み込み
  - .surql ファイル形式でのエクスポート/インポート
  - 起動時の自動インポート、定期的な自動エクスポート

- **`src/ci/`**: GitHub Actions CI監視
  - `CiMonitor` - gh CLIを使用したCI/CD パイプライン監視
  - ビルド状態のポーリング、失敗ログ取得

- **`src/web/`**: Webダッシュボードサーバー
  - `server.rs`: HTTP サーバー実装（自動ポート選択機能付き）
  - デフォルトポート 12700、占有時は自動で別ポートを選択

- **`static/`**: Webダッシュボードアセット
  - `index.html`: Alpine.js と Tabler UI を使用したSPA

### 主要な設計パターン

1. **Arc<RwLock> パターン**: `ProcessManager` は管理対象プロセスへのスレッドセーフな並行アクセスのために `Arc<RwLock<HashMap>>` を使用。各プロセスも細かいロック制御のために `Arc<RwLock>` でラップされています。

2. **ステートマシン**: プロセスは状態を遷移します：`NotStarted` → `Running` → `Stopped`/`Failed`。状態遷移はアトミックで、タイムスタンプを含みます。

3. **非同期出力キャプチャ**: プロセス開始時、stdout/stderr を循環バッファにキャプチャする2つの非同期タスクが生成され、長時間実行プロセスによるメモリ枯渇を防ぎます。

4. **ツールルーター**: `#[tool_router]` マクロが MCP ツールルーティングを生成。ツールは `CallToolResult` を返す非同期関数です。

5. **永続化アーキテクチャ**: SurrealDB インメモリエンジン（kv-mem）を使用。プロセス定義は `UPDATE` クエリで保存され、配列やオブジェクトが適切に保持されます。

## MCP 統合ポイント

サーバーは以下のツールを Claude に公開します：
- 基本: `echo`、`ping`、`get_status`
- プロセス管理: `create_process`、`start_process`、`stop_process`、`get_process_status`、`get_process_output`、`list_processes`、`remove_process`
- 永続化: `export_processes`、`import_processes`
- CI監視: `list_ci_runs`、`get_ci_run_details`、`get_ci_failed_logs`、`wait_for_ci_completion`、`start_ci_monitoring`

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

## 環境変数

| 変数 | 説明 | デフォルト |
|------|------|------------|
| `RUST_LOG` | ログレベル (error, warn, info, debug, trace) | info |
| `ICHIMI_AUTO_EXPORT_INTERVAL` | 自動エクスポート間隔（秒） | なし |
| `ICHIMI_IMPORT_FILE` | 起動時にインポートするファイル | ~/.ichimi/data/processes.surql |
| `ICHIMI_EXPORT_FILE` | シャットダウン時のエクスポート先 | ~/.ichimi/data/processes.surql |
| `ICHIMI_DATA_DIR` | データファイル用ディレクトリ | ~/.ichimi/data |
| `ICHIMI_STOP_ON_SHUTDOWN` | ichimi終了時にプロセスを停止するか（true/false） | false（継続） |

## テストに関する考慮事項

テストは `tests/` ディレクトリに配置されています：
- `test_persistence.rs`: 永続化層のユニットテスト
- `test_process_manager.rs`: ProcessManager の統合テスト

テスト実行：
```bash
cargo test                    # 全テストを実行
cargo test test_export_import # 特定のテストを実行
```

## 開発のヒント

1. **新しいMCPツールを追加する場合**：
   - `src/messages/process.rs` にリクエスト型を定義
   - `src/lib.rs` の `IchimiServer` impl ブロックにツールメソッドを追加
   - `#[tool]` 属性でメソッドを装飾

2. **プロセス管理ロジックを変更する場合**：
   - `src/process/manager.rs` の `ProcessManager` を更新
   - 状態遷移は `ProcessState` enum で定義

3. **永続化を変更する場合**：
   - `src/persistence.rs` の `PersistenceManager` を更新
   - SurrealDB スキーマ定義に注意

## リリース手順

### バージョン管理とリリースの作成

1. **Cargo.tomlのバージョン更新**（重要）:
   ```bash
   # Cargo.tomlのversionフィールドを更新
   # 例: version = "0.1.0-beta11" → version = "0.1.0-beta12"
   ```

2. **ビルドとテスト**:
   ```bash
   # バージョン更新後、必ずビルドとテストを実行
   cargo build --release
   cargo test
   ```

3. **コミットとタグ作成**:
   ```bash
   # Cargo.tomlの変更をコミット
   git add Cargo.toml Cargo.lock
   git commit -m "chore: bump version to v0.1.0-betaXX"
   
   # タグを作成
   git tag -a v0.1.0-betaXX -m "Release v0.1.0-betaXX - 簡潔な説明"
   git push origin main
   git push origin v0.1.0-betaXX
   ```

4. **GitHubリリースの作成**:
   ```bash
   gh release create v0.1.0-betaXX \
     --title "v0.1.0-betaXX - タイトル" \
     --notes-file release-notes.md \
     --prerelease
   ```

### 重要な注意事項
- **必ずCargo.tomlのバージョンを更新してからリリースすること**
- バージョン番号はCargo.tomlとGitタグで一致させる
- cargo installコマンドが正しく動作するよう、タグ名は`v`プレフィックスを付ける