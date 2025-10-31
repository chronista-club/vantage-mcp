# CLAUDE.md

このファイルは、このリポジトリのコードを扱う際の Claude Code (claude.ai/code) へのガイダンスを提供します。

**重要**: このプロジェクトのメイン言語は日本語です。コミットメッセージ、ドキュメント、コメントは日本語で記述してください。

## プロジェクト概要

Vantage MCP Server - Model Context Protocol (MCP) を介したプロセス管理サーバー。Claude Code がプロセスの起動、停止、監視、ログキャプチャを可能にします。

## ビルド・実行コマンド

### Cargo コマンド
```bash
# ビルド
cargo build           # デバッグビルド
cargo build --release # リリースビルド（最適化済み）

# テスト実行
cargo test                     # 全テストを実行
cargo test test_name          # 特定のテストを実行
cargo test -- --nocapture     # テスト出力を表示

# コード品質チェック
cargo fmt             # コードフォーマット
cargo fmt -- --check  # フォーマットチェックのみ
cargo clippy          # リンターを実行
cargo clippy -- -D warnings  # 警告をエラーとして扱う

# サーバー実行
cargo run --bin vantagemcp            # MCPサーバーを起動
cargo run --bin vantagemcp -- --no-open  # ブラウザを開かずに起動
./target/release/vantagemcp          # リリースビルドを実行
```

### Mise タスク（推奨）
```bash
# 開発コマンド
mise run dev          # Web UIビルド後、サーバー起動
mise run dev-watch    # Rustコード自動リロード付き開発サーバー
mise run web-watch    # Web UIのみビルド監視
mise run web          # Webダッシュボードモードで起動

# ビルド・インストール
mise run build        # リリースビルド（Web UI含む）
mise run build-debug  # デバッグビルド
mise run install      # ~/.local/bin にインストール
mise run install-debug # デバッグビルドをインストール

# その他
mise run test         # テスト実行
mise run clean        # ビルド成果物削除
```

### Web UI 開発（Vue 3）
```bash
cd ui/web
bun install           # 依存関係インストール
bun run dev           # 開発サーバー起動（ポート 5173）
bun run build         # プロダクションビルド
```

### 環境変数
```bash
RUST_LOG=debug cargo run                    # デバッグログ有効
VANTAGE_AUTO_EXPORT_INTERVAL=300 cargo run  # 5分ごとに自動エクスポート
VANTAGE_STOP_ON_SHUTDOWN=true cargo run     # 終了時にプロセス停止
```

## アーキテクチャ概要

### クレート構造
- **crates/vantage**: メインサーバー実装
  - MCP ツールハンドラー（`#[tool]`属性）
  - プロセス管理（Arc<RwLock>パターン）
  - 循環バッファによるログ管理

- **crates/vantage-persistence**: 永続化層
  - YAMLスナップショット（`.vantage/snapshot.yaml`）
  - インメモリストレージ

- **crates/vantage-mcp**: MCPサーバーバイナリ
  - エントリーポイント
  - CLIオプション処理

### 主要コンポーネント
1. **ProcessManager**: プロセスライフサイクル管理
   - 状態遷移: NotStarted → Running → Stopped/Failed
   - SIGTERM送信後の猶予期間付き停止
   - プロセスグループ管理（Dockerコンテナ対応）

2. **CircularBuffer**: メモリ効率的なログストレージ
   - 固定容量でOOMを防止
   - stdout/stderr の非同期キャプチャ

3. **Web Dashboard**: Vue 3 SPA
   - Tabler UIフレームワーク
   - リアルタイム更新
   - デフォルトポート 12700（自動フォールバック機能付き）

## MCP ツール一覧

- 基本: `echo`, `ping`, `get_status`
- プロセス管理: `create_process`, `start_process`, `stop_process`, `get_process_status`, `get_process_output`, `list_processes`, `remove_process`
- 永続化: `export_processes`, `import_processes`
- CI監視: `list_ci_runs`, `get_ci_run_details`, `get_ci_failed_logs`

## 開発時の注意点

### プロセス管理
- Dockerプロセスを扱う場合、プロセスグループを適切に設定
- 終了時は SIGTERM → 猶予期間 → SIGKILL の順序
- 循環バッファのサイズに注意（デフォルト 10000行）

### テスト実行
- 統合テスト: `crates/vantage/tests/`
- 単体テスト: 各モジュール内の `#[cfg(test)]`
- E2Eテスト: ブラウザ関連機能を含む

### リリース手順
1. Cargo.toml のバージョン更新
2. `cargo build --release && cargo test`
3. Git タグ作成: `git tag -a v0.x.x -m "Release v0.x.x"`
4. GitHub Release 作成: `gh release create`

## プロジェクト固有の規約

- エラーハンドリング: `Result<T, String>` を返し、MCP エラーに変換
- 非同期処理: Tokio ランタイムを使用
- ログ出力: `tracing` クレートを使用（`info!`, `debug!`, `error!`）
- コミットメッセージ: 日本語で記述、conventional commits 形式