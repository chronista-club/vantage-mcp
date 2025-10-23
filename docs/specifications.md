# Vantage Server 技術仕様書

## プロジェクト概要

### 基本情報

| 項目 | 内容 |
|------|------|
| プロジェクト名 | Vantage Server（一味サーバー） |
| バージョン | 0.2.3 |
| リポジトリ | https://github.com/chronista-club/vantage-server |
| ライセンス | MIT OR Apache-2.0 |
| 作者 | mito@chronista.club |
| 言語 | Rust 2024 Edition |

### コンセプト

**"Process as a Resource"** - プロセスをリソースとして管理する

Vantage Serverは、Model Context Protocol (MCP) を介してClaude Codeと連携し、プロセスの起動・監視・管理を行う統合プロセス管理サーバーです。開発者がClaude Codeとの対話を通じて、ローカルプロセスを自然言語で制御できる環境を提供します。

### 主要機能

1. **プロセス管理**: 任意のプロセスの起動/停止/監視
2. **リアルタイムログ**: stdout/stderrのキャプチャとストリーミング
3. **Webダッシュボード**: Vue 3ベースのモダンなUI
4. **MCP統合**: Claude Code完全対応
5. **永続化**: YAML形式でのスナップショット管理
6. **CI/CD監視**: GitHub Actions統合（gh CLI使用）
7. **クリップボード管理**: テキスト・コードスニペット管理
8. **テンプレート機能**: プロセス設定のテンプレート化

## システム要件

### 動作環境

#### サポートOS
- ✅ macOS (M1/M2/Intel)
- ✅ Linux (x86_64, ARM64)
- ⚠️ Windows (実験的サポート)

#### 必須要件
- Rust 1.75+ (ビルド時)
- メモリ: 最小 256MB、推奨 512MB以上
- ディスク: 最小 100MB、推奨 1GB以上

#### オプション要件
- Node.js 18+ / Bun (Web UIビルド時)
- gh CLI (GitHub Actions監視機能使用時)
- Docker (コンテナプロセス管理時)

### 依存関係

#### Rustクレート（主要）

**コア:**
- `tokio` (1.45): 非同期ランタイム
- `rmcp` (0.7.0): MCP SDK
- `serde` (1.0): シリアライゼーション
- `anyhow` (1.0): エラーハンドリング

**Web:**
- `axum` (0.7): Webフレームワーク
- `tower` / `tower-http`: HTTPミドルウェア
- `rust-embed`: 静的ファイル埋め込み

**永続化:**
- `serde_yaml`: YAML永続化
- `dirs`: ディレクトリ管理

**その他:**
- `chrono`: 日時処理
- `tracing`: ロギング
- `reqwest`: HTTPクライアント
- `nix` (Unix): シグナル処理

#### JavaScript/TypeScript（Web UI）

- Vue 3
- TypeScript
- Vite
- Pinia
- Vue Router
- Tabler UI

## アーキテクチャ

### システム構成

```
┌─────────────────────────────────────────────┐
│           Claude Code / MCP Client          │
└─────────────┬───────────────────────────────┘
              │ MCP Protocol (stdio)
┌─────────────┴───────────────────────────────┐
│         Vantage Server (Rust)                │
├─────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────┐ │
│ │     MCP Server (rmcp)                   │ │
│ │  - Tool Handlers                        │ │
│ │  - Request/Response Processing          │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ ┌─────────────┐  ┌─────────────────────┐   │
│ │ Process     │  │ Web Server          │   │
│ │ Manager     │  │ (Axum + Embedded)   │   │
│ │             │  │                     │   │
│ │ - Lifecycle │  │ - REST API          │   │
│ │ - Logging   │  │ - Static Files      │   │
│ │ - Status    │  │ - WebSocket (TODO)  │   │
│ └─────────────┘  └─────────────────────┘   │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │  Persistence Manager                    │ │
│ │  - In-memory Storage (Arc<RwLock>)      │ │
│ │  - YAML Snapshot Export/Import          │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
              │
              ▼
    ┌──────────────────┐
    │  File System     │
    │  ~/.vantage/      │
    │   └─ data/       │
    │      └─ snapshot.yaml │
    └──────────────────┘
```

### データフロー

#### プロセス起動フロー

```
Claude Code
  │
  ├─ MCP Request: start_process(id)
  │
  ▼
Vantage Server (MCP Handler)
  │
  ├─ ProcessManager::start_process()
  │   │
  │   ├─ tokio::process::Command::spawn()
  │   │
  │   ├─ Capture stdout/stderr
  │   │   │
  │   │   └─ CircularBuffer (非同期タスク)
  │   │
  │   └─ Update ProcessState → Running
  │
  └─ MCP Response: { success: true }
```

#### Web UIからのプロセス操作

```
Browser
  │
  ├─ HTTP POST /api/processes/:id/start
  │
  ▼
Axum Web Server
  │
  ├─ API Handler
  │   │
  │   └─ ProcessManager::start_process()
  │       │
  │       └─ (プロセス起動フロー)
  │
  └─ HTTP 200 OK
```

## プロセス管理仕様

### プロセス状態遷移

```
NotStarted ──start──▶ Running ──stop──▶ Stopped
                         │
                         └──error──▶ Failed
```

**状態定義:**

| 状態 | 説明 | データ |
|------|------|--------|
| `NotStarted` | プロセス未起動 | なし |
| `Running` | プロセス実行中 | `{ pid, started_at }` |
| `Stopped` | プロセス正常終了 | `{ exit_code?, stopped_at }` |
| `Failed` | プロセス異常終了 | `{ error, failed_at }` |

### プロセスライフサイクル

#### 1. 作成 (`create_process`)

```rust
CreateProcessRequest {
    id: String,              // プロセスID (一意)
    command: String,         // 実行コマンド
    args: Vec<String>,       // 引数リスト
    env: HashMap<String, String>, // 環境変数
    cwd: Option<String>,     // 作業ディレクトリ
    auto_start_on_restore: bool, // リストア時の自動起動
}
```

**セキュリティ検証:**
- コマンドインジェクション防止
- パストラバーサル防止
- 環境変数サニタイゼーション

#### 2. 起動 (`start_process`)

```rust
async fn start_process(id: String) -> Result<()> {
    // 1. プロセス設定取得
    // 2. tokio::process::Command生成
    // 3. stdout/stderr非同期キャプチャ開始
    // 4. PID記録
    // 5. 状態をRunningに更新
}
```

**実装詳細:**
- プロセスグループ設定（Unix）: `process_group(0)`
- シグナル送信: SIGTERM → 猶予期間 → SIGKILL

#### 3. 停止 (`stop_process`)

```rust
async fn stop_process(id: String) -> Result<()> {
    // 1. Running状態確認
    // 2. SIGTERM送信（Unix）/ TerminateProcess（Windows）
    // 3. 5秒間待機
    // 4. タイムアウト時: SIGKILL送信
    // 5. 状態をStoppedに更新
}
```

#### 4. 削除 (`remove_process`)

- プロセスが実行中の場合: エラー（先にstopが必要）
- 状態管理から削除
- ログバッファクリア

### ログ管理

#### CircularBuffer

固定容量の循環バッファでメモリ効率化：

```rust
struct CircularBuffer {
    capacity: usize,      // デフォルト 10,000行
    lines: VecDeque<String>,
}
```

**特徴:**
- メモリ消費量固定
- 古いログは自動削除
- スレッドセーフ（Arc<RwLock>）

#### ログ取得

```bash
GET /api/processes/:id/logs?stream=stdout&lines=100
```

**パラメータ:**
- `stream`: `stdout` or `stderr`
- `lines`: 取得行数（デフォルト 100）

## 永続化仕様

### インメモリストレージ

```rust
Arc<RwLock<HashMap<String, ProcessConfig>>>
```

**特徴:**
- 高速アクセス
- スレッドセーフ
- 再起動で消失（スナップショット必須）

### スナップショット機能

#### エクスポート

```yaml
# ~/.vantage/data/snapshot.yaml
processes:
  example-process:
    id: example-process
    command: echo
    args: ["Hello, World"]
    env:
      FOO: bar
    cwd: /path/to/dir
    auto_start_on_restore: true
    state: NotStarted
```

**自動エクスポート:**
- 環境変数 `VANTAGE_AUTO_EXPORT_INTERVAL`: 自動エクスポート間隔（秒）
- シャットダウン時の自動エクスポート

#### インポート

起動時オプション:
```bash
vantage --import ~/.vantage/data/snapshot.yaml
```

**auto_start_on_restore:**
- `true`: インポート時に自動起動
- `false`: 起動しない

## MCP統合仕様

### MCP Tools

#### 基本ツール

| ツール | 説明 |
|--------|------|
| `echo` | エコーテスト |
| `ping` | ヘルスチェック |
| `get_status` | サーバー状態取得 |

#### プロセス管理ツール

| ツール | 説明 |
|--------|------|
| `create_process` | プロセス作成 |
| `start_process` | プロセス起動 |
| `stop_process` | プロセス停止 |
| `get_process_status` | プロセス状態取得 |
| `get_process_output` | ログ取得 |
| `list_processes` | プロセス一覧 |
| `remove_process` | プロセス削除 |

#### 永続化ツール

| ツール | 説明 |
|--------|------|
| `export_processes` | スナップショットエクスポート |
| `import_processes` | スナップショットインポート |

#### CI/CD監視ツール

| ツール | 説明 |
|--------|------|
| `list_ci_runs` | CIラン一覧 |
| `get_ci_run_details` | CIラン詳細 |
| `get_ci_failed_logs` | 失敗ログ取得 |
| `wait_for_ci_completion` | CI完了待機 |
| `start_ci_monitoring` | CI監視開始 |

### MCP Protocol

**Transport:** stdio

**Format:** JSON-RPC 2.0

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "start_process",
    "arguments": {
      "id": "my-process"
    }
  }
}
```

## Web API仕様

### ベースURL

```
http://localhost:12700
```

### エンドポイント

#### プロセス管理

**GET /api/processes**
- 説明: プロセス一覧取得
- レスポンス: `ProcessInfo[]`

**POST /api/processes**
- 説明: プロセス作成
- ボディ: `CreateProcessRequest`
- レスポンス: `201 Created`

**POST /api/processes/:id/start**
- 説明: プロセス起動
- レスポンス: `200 OK`

**POST /api/processes/:id/stop**
- 説明: プロセス停止
- レスポンス: `200 OK`

**DELETE /api/processes/:id**
- 説明: プロセス削除
- レスポンス: `204 No Content`

**GET /api/processes/:id/logs**
- 説明: ログ取得
- クエリパラメータ:
  - `stream`: `stdout` | `stderr`
  - `lines`: 取得行数
- レスポンス: `string[]`

#### ダッシュボード

**GET /api/dashboard**
- 説明: ダッシュボードデータ
- レスポンス: 統計情報 + プロセス一覧

#### クリップボード

**GET /api/clipboard**
**POST /api/clipboard**
**DELETE /api/clipboard/:id**

## パフォーマンス仕様

### リソース消費

#### メモリ

| コンポーネント | メモリ使用量 |
|----------------|--------------|
| サーバー本体 | 約 10-20MB |
| プロセスあたり | 約 1-2MB |
| ログバッファ（10,000行） | 約 1-5MB |
| Web UI（埋め込み） | 約 5MB |

**推奨上限:**
- 同時起動プロセス: 50-100
- 総メモリ使用量: 500MB以下

#### CPU

- アイドル時: 0-1%
- プロセス起動/停止時: 1-5%
- ログキャプチャ: プロセスあたり 1-2%

### レスポンス時間

| 操作 | 目標レスポンスタイム |
|------|----------------------|
| プロセス作成 | < 10ms |
| プロセス起動 | < 100ms |
| プロセス停止 | < 5秒 |
| ログ取得 | < 50ms |
| API呼び出し | < 100ms |

### スケーラビリティ

**水平スケーリング:**
- 現状非対応（シングルインスタンス）
- 将来: マルチインスタンス + 分散状態管理

**垂直スケーリング:**
- メモリ増設で同時プロセス数増加可能
- CPU増設で並列処理性能向上

## セキュリティ仕様

### 入力検証

#### コマンドインジェクション対策

```rust
// 禁止文字
const DANGEROUS_CHARS: &[char] = &['|', '&', ';', '\n', '`', '$'];

fn validate_command(cmd: &str) -> Result<()> {
    if cmd.contains(DANGEROUS_CHARS) {
        return Err("Dangerous characters in command");
    }
    Ok(())
}
```

#### パストラバーサル対策

```rust
fn validate_working_directory(path: &str) -> Result<()> {
    if path.contains("..") {
        return Err("Path traversal detected");
    }
    Ok(())
}
```

#### 環境変数サニタイゼーション

```rust
fn sanitize_env(env: &HashMap<String, String>) -> HashMap<String, String> {
    env.iter()
        .filter(|(k, _)| is_safe_env_key(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}
```

### 認証・認可

**現状:**
- ローカルホストのみアクセス可能
- 認証機能なし（ローカル開発環境想定）

**将来:**
- トークンベース認証
- RBAC（ロールベース アクセス制御）
- TLS対応

### プロセス分離

- 各プロセスは独立したコンテキストで実行
- プロセス間の干渉なし
- リソース制限（TODO: cgroups対応）

## 運用仕様

### 起動オプション

```bash
vantage [OPTIONS]
```

**オプション:**
- `--web`: Webダッシュボードを有効化
- `--web-port <PORT>`: Webサーバーポート指定（デフォルト 12700）
- `--import <FILE>`: スナップショットインポート
- `--export <FILE>`: スナップショットエクスポート

### 環境変数

| 変数 | 説明 | デフォルト |
|------|------|------------|
| `RUST_LOG` | ログレベル | `info` |
| `VANTAGE_AUTO_EXPORT_INTERVAL` | 自動エクスポート間隔（秒） | なし |
| `VANTAGE_DATA_DIR` | データディレクトリ | `~/.vantage/data` |
| `VANTAGE_STOP_ON_SHUTDOWN` | 終了時のプロセス停止 | `false` |

### ログ出力

**レベル:**
- `error`: エラー
- `warn`: 警告
- `info`: 情報（デフォルト）
- `debug`: デバッグ
- `trace`: トレース

**出力先:**
- stdout（構造化ログ）

### シャットダウン

**グレースフルシャットダウン:**
1. SIGTERM/SIGINT受信
2. 自動エクスポート実行
3. オプション: 全プロセス停止（`VANTAGE_STOP_ON_SHUTDOWN=true`）
4. クリーンアップ
5. 終了

## テスト仕様

### ユニットテスト

**カバレッジ目標:** 70%以上

**テスト対象:**
- ProcessManager
- PersistenceManager
- セキュリティ検証
- 状態遷移

### 統合テスト

**テストスイート:**
- `test_web_api.rs`: Web APIテスト（4テスト）
- `test_process_manager.rs`: プロセス管理テスト
- `test_browser_mcp.rs`: E2Eテスト（Chrome MCP）

**実行:**
```bash
cargo test
cargo test --test test_web_api --features web
```

### E2Eテスト

**Chrome MCP DevTools:**
- ダッシュボード表示
- プロセス作成・起動・停止
- ログ表示
- スクリーンショット検証

## ビルド仕様

### リリースビルド

```bash
cargo build --release
```

**最適化設定:**
```toml
[profile.release]
opt-level = 3           # 最大最適化
lto = true              # Link Time Optimization
codegen-units = 1       # 単一コード生成ユニット
strip = true            # デバッグシンボル削除
```

**バイナリサイズ:** 約 15-20MB（Web UI埋め込み込み）

### Web UIビルド

```bash
cd ui/web
bun install
bun run build
```

**出力:** `ui/web/dist/`

**埋め込み:** `rust-embed`でバイナリに埋め込み

## バージョニング

**形式:** Semantic Versioning (SemVer)

```
MAJOR.MINOR.PATCH[-PRERELEASE]
```

**例:**
- `0.2.3`: 安定版
- `0.3.0-beta1`: ベータ版
- `1.0.0`: メジャーリリース

## 今後のロードマップ

### v0.3.0（予定）
- [ ] WebSocket対応（リアルタイムログストリーミング）
- [ ] プロセスグループ管理
- [ ] ログ表示モーダル実装

### v0.4.0（予定）
- [ ] 認証機能
- [ ] マルチユーザー対応
- [ ] プラグインシステム

### v1.0.0（目標）
- [ ] 完全な安定性
- [ ] 包括的ドキュメント
- [ ] エンタープライズ対応

## ライセンス

MIT OR Apache-2.0

詳細: [LICENSE-MIT](../LICENSE-MIT) / [LICENSE-APACHE](../LICENSE-APACHE)
