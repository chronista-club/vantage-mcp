# CLAUDE.md

このファイルは、このリポジトリのコードを扱う際の Claude Code (claude.ai/code) へのガイダンスを提供します。

## プロジェクト概要

Vantage (一味・いちみ) Server は Model Context Protocol (MCP) を介した Claude Code 用のプロセス管理サーバーです。Claude がプロセスの起動、停止、監視、および MCP ツールを通じた出力のキャプチャを可能にします。

### 主な機能
- プロセスのライフサイクル管理（作成、起動、停止、削除）
- リアルタイムログキャプチャ（stdout/stderr）
- YAML形式での永続化と設定ファイル管理（.vantage/snapshot.yaml）
- Webダッシュボード（Vue 3 + TypeScript + Vite + Tabler UI）
- 自動バックアップ機能

## インストール方法

```bash
# GitHubから特定のバージョンを直接インストール（推奨）
cargo install --git https://github.com/chronista-club/vantage-server --tag v0.1.0-beta20

# 最新のmainブランチからインストール
cargo install --git https://github.com/chronista-club/vantage-server

# ローカルでビルドしてインストール
git clone https://github.com/chronista-club/vantage-server.git
cd vantage-server
cargo install --path crates/vantage
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
cargo run --bin vantage
./target/release/vantage # リリースビルドを実行

# Webダッシュボード付きで実行
cargo run --bin vantage -- --web
cargo run --bin vantage -- --web --web-port 8080  # カスタムポート

# 環境変数を設定して実行
RUST_LOG=debug cargo run
VANTAGE_AUTO_EXPORT_INTERVAL=300 cargo run  # 5分ごとに自動エクスポート
```

## アーキテクチャ

### モジュール構造

プロジェクトはワークスペース構造で整理されています：

### crates/vantage - メインサーバークレート

- **`src/lib.rs`**: MCP ツールハンドラーを持つメインサーバー実装。各ツールメソッドは `#[tool]` 属性で装飾され、Claude に公開される MCP ツールにマッピングされます。

- **`src/messages/`**: リクエスト/レスポンスメッセージ構造体
  - `basic.rs`: シンプルなメッセージタイプ（echo、ping）
  - `process.rs`: プロセス管理リクエストタイプ（内部型）
  - `ci.rs`: CI監視リクエストタイプ
  - `clipboard.rs`: クリップボード管理タイプ
  
- **`src/process/`**: コアプロセス管理ロジック
  - `manager.rs`: `ProcessManager` - プロセスライフサイクルを処理
  - `buffer.rs`: `CircularBuffer` - 固定容量でメモリ効率的なログストレージ
  - `protocol.rs`: プロセスプロトコル定義
  - `shell.rs`: シェル統合

- **`src/ci/`**: GitHub Actions CI監視
  - `CiMonitor` - gh CLIを使用したCI/CD パイプライン監視
  - ビルド状態のポーリング、失敗ログ取得

- **`src/web/`**: Webダッシュボードサーバー
  - `server.rs`: HTTP サーバー実装（自動ポート選択機能付き）
  - `handlers.rs`: APIハンドラー実装
  - `api.rs`: APIルーティング
  - デフォルトポート 12700、占有時は自動で別ポートを選択

### crates/vantage-persistence - 永続化レイヤー

- **`src/lib.rs`**: 永続化インターフェース定義
- **`src/persistence/`**: インメモリストレージとYAML永続化実装
  - `Arc<RwLock<HashMap>>`による高速メモリストレージ
  - プロセス、クリップボード、設定の管理
  - YAMLスナップショットのエクスポート/インポート機能
  - バックアップとリストア

### ui/web - Vue 3 SPA

- **モダンなフロントエンドスタック**:
  - Vue 3 + TypeScript + Vite
  - Pinia によるステート管理
  - Vue Router によるルーティング
  - Tabler UI フレームワーク
  - Bun パッケージマネージャー

- **主要ディレクトリ**:
  - `src/components/`: 再利用可能なVueコンポーネント（SFC）
  - `src/views/`: ページレベルコンポーネント
  - `src/stores/`: Piniaステート管理
  - `src/api/`: APIクライアント層
  - `src/types/`: TypeScript型定義
  - `src/themes.ts`: Vantage Design System（OKLCH色空間）

### 主要な設計パターン

1. **Arc<RwLock> パターン**: `ProcessManager` は管理対象プロセスへのスレッドセーフな並行アクセスのために `Arc<RwLock<HashMap>>` を使用。各プロセスも細かいロック制御のために `Arc<RwLock>` でラップされています。

2. **ステートマシン**: プロセスは状態を遷移します：`NotStarted` → `Running` → `Stopped`/`Failed`。状態遷移はアトミックで、タイムスタンプを含みます。

3. **非同期出力キャプチャ**: プロセス開始時、stdout/stderr を循環バッファにキャプチャする2つの非同期タスクが生成され、長時間実行プロセスによるメモリ枯渇を防ぎます。

4. **ツールルーター**: `#[tool_router]` マクロが MCP ツールルーティングを生成。ツールは `CallToolResult` を返す非同期関数です。

5. **永続化アーキテクチャ**:
   - YAML形式: 人間が読み書きしやすい設定ファイル形式
   - インメモリストレージ: `Arc<RwLock<HashMap>>`による高速アクセス
   - スナップショット機能: エクスポート/インポート機能とバックアップ

## MCP 統合ポイント

サーバーは以下のツールを Claude に公開します：
- 基本: `echo`、`ping`、`get_status`
- プロセス管理: `create_process`、`start_process`、`stop_process`、`get_process_status`、`get_process_output`、`list_processes`、`remove_process`
- 永続化: `export_processes`、`import_processes`
- CI監視: `list_ci_runs`、`get_ci_run_details`、`get_ci_failed_logs`、`wait_for_ci_completion`、`start_ci_monitoring`

各ツールは `lib.rs` の `VantageServer` impl ブロック内のメソッドに直接マッピングされます。

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
| `VANTAGE_AUTO_EXPORT_INTERVAL` | 自動エクスポート間隔（秒） | なし |
| `VANTAGE_IMPORT_FILE` | 起動時にインポートするファイル | ~/.vantage/data/processes.surql |
| `VANTAGE_EXPORT_FILE` | シャットダウン時のエクスポート先 | ~/.vantage/data/processes.surql |
| `VANTAGE_DATA_DIR` | データファイル用ディレクトリ | ~/.vantage/data |
| `VANTAGE_STOP_ON_SHUTDOWN` | vantage終了時にプロセスを停止するか（true/false） | false（継続） |

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
   - `crates/vantage/src/messages/` にリクエスト型を定義
   - `crates/vantage/src/lib.rs` の `VantageServer` impl ブロックにツールメソッドを追加
   - `#[tool]` 属性でメソッドを装飾

2. **プロセス管理ロジックを変更する場合**：
   - `crates/vantage/src/process/manager.rs` の `ProcessManager` を更新
   - 内部型は `crates/vantage/src/messages/process.rs` で定義

3. **永続化を変更する場合**：
   - `crates/vantage-persistence/src/` の該当モジュールを更新
   - YAML形式でのスナップショット機能

4. **WebUI開発**：
   ```bash
   cd ui/web-vue
   bun install        # 依存関係インストール
   bun run dev        # 開発サーバー起動（ポート 5173）
   bun run build      # プロダクションビルド → ui/web/dist
   ```
   
   - Vue 3 SFCでコンポーネントを開発
   - TypeScriptで完全な型安全性を確保
   - Pinia storeで状態管理
   - ui/web/distにビルド出力（Rustから配信）

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