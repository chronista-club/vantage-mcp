# Ichimi Server - プロセス管理MCPサーバー

## 概要

Ichimi Serverは、Model Context Protocol (MCP) を介してClaude Codeと統合される強力なプロセス管理サーバーです。開発環境でのプロセス管理を効率化し、ログ監視、状態追跡、ライフサイクル管理を統合的に提供します。

## 主な特徴

- 🚀 **プロセス管理**: あらゆるプロセスの起動、停止、監視
- 📊 **リアルタイムログ**: stdout/stderrの出力をキャプチャしてストリーミング
- 🔍 **ステータス監視**: プロセスの状態とメトリクスを追跡
- 🎯 **柔軟なフィルタリング**: プロセスのリストと検索
- 💾 **メモリ効率**: 循環バッファによる効率的なログ管理
- 🔌 **Claude Code統合**: MCPによるネイティブ統合

## インストール

### 方法1: ソースからのインストール

```bash
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server
cargo build --release
# バイナリは target/release/ichimi-server に生成されます
```

### 方法2: Cargo経由でのインストール

```bash
cargo install ichimi-server
```

## 設定

### .mcp.json設定

プロジェクトルートの`.mcp.json`に以下を追加：

```json
{
  "mcpServers": {
    "ichimi": {
      "type": "stdio",
      "command": "ichimi-server",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 環境変数

- `RUST_LOG`: ログレベル設定（`error`, `warn`, `info`, `debug`, `trace`）
- `ICHIMI_MAX_LOG_LINES`: プロセスごとの最大ログ行数（デフォルト: 1000）

## 利用可能なツール

### 基本ツール

#### echo
テスト用のメッセージエコーバック
```typescript
mcp__ichimi-server__echo({
  message: "テストメッセージ"
})
```

#### ping
ヘルスチェック（"pong"を返す）
```typescript
mcp__ichimi-server__ping()
```

#### get_status
サーバーステータスと稼働時間を取得
```typescript
mcp__ichimi-server__get_status()
// 返り値: { status: "running", version: "0.1.0", uptime: 123 }
```

### プロセス管理ツール

#### create_process
新しいプロセス設定を登録
```typescript
mcp__ichimi-server__create_process({
  id: "my-server",
  command: "cargo",
  args: ["run", "--release"],
  cwd: "/path/to/project",
  env: {
    "PORT": "3000",
    "RUST_LOG": "info"
  }
})
```

#### start_process
登録済みプロセスを起動
```typescript
mcp__ichimi-server__start_process({
  id: "my-server"
})
// 返り値: { pid: 12345 }
```

#### stop_process
プロセスを正常停止（グレースフルシャットダウン）
```typescript
mcp__ichimi-server__stop_process({
  id: "my-server",
  grace_period_ms: 5000  // オプション: 強制終了までの待機時間
})
```

#### get_process_status
プロセスの詳細ステータスを取得
```typescript
mcp__ichimi-server__get_process_status({
  id: "my-server"
})
// 返り値: {
//   info: { id, command, args, env, cwd, state },
//   cpu_usage: 2.5,
//   memory_usage: 150000000,
//   uptime_seconds: 3600
// }
```

#### get_process_output
プロセスのログを取得
```typescript
mcp__ichimi-server__get_process_output({
  id: "my-server",
  stream: "Both",  // "Stdout", "Stderr", "Both"
  lines: 50  // オプション: 取得する行数
})
```

#### list_processes
プロセス一覧を取得（フィルタリング可能）
```typescript
mcp__ichimi-server__list_processes({
  filter: {
    state: "Running",  // "Running", "Stopped", "Failed", "All"
    name_pattern: "server"  // オプション: 名前パターン
  }
})
```

#### remove_process
プロセスを管理対象から削除
```typescript
mcp__ichimi-server__remove_process({
  id: "my-server"
})
```

## 使用例

### 開発サーバーの管理

```typescript
// 1. プロセスを作成
await mcp__ichimi-server__create_process({
  id: "diarkis-devtools",
  command: "cargo",
  args: ["run", "--release", "--manifest-path", "apps/viewer-rs/Cargo.toml"],
  cwd: "/Users/mito/Workspaces/DIARKIS/diarkis-tools",
  env: {
    "VIEWER_PORT": "31279",
    "RUST_LOG": "info"
  }
})

// 2. プロセスを起動
await mcp__ichimi-server__start_process({
  id: "diarkis-devtools"
})

// 3. ログを監視
const logs = await mcp__ichimi-server__get_process_output({
  id: "diarkis-devtools",
  stream: "Both",
  lines: 100
})

// 4. ステータス確認
const status = await mcp__ichimi-server__get_process_status({
  id: "diarkis-devtools"
})

// 5. 停止（必要時）
await mcp__ichimi-server__stop_process({
  id: "diarkis-devtools",
  grace_period_ms: 3000
})
```

### 複数プロセスの管理

```typescript
// すべてのプロセスをリスト
const processes = await mcp__ichimi-server__list_processes()

// 実行中のプロセスのみ
const running = await mcp__ichimi-server__list_processes({
  filter: { state: "Running" }
})

// 特定パターンのプロセス
const servers = await mcp__ichimi-server__list_processes({
  filter: { name_pattern: "server" }
})
```

## プロセス状態

- **Starting**: 起動中
- **Running**: 実行中
- **Stopping**: 停止中
- **Stopped**: 停止済み
- **Failed**: エラーで停止

## ベストプラクティス

1. **プロセスIDの命名規則**
   - 意味のある名前を使用（例: `web-server`, `db-backup`）
   - ハイフンで単語を区切る
   - 小文字を使用

2. **環境変数の管理**
   - センシティブな情報は環境変数で管理
   - プロセスごとに適切な環境を設定

3. **ログ管理**
   - 適切なログレベルを設定
   - 定期的にログを確認

4. **グレースフルシャットダウン**
   - 適切な`grace_period_ms`を設定
   - SIGTERMを適切に処理するようアプリケーションを設計

5. **エラーハンドリング**
   - プロセスの状態を定期的に確認
   - Failed状態のプロセスを適切に処理

## トラブルシューティング

### プロセスが起動しない
- コマンドとパスが正しいか確認
- 必要な環境変数が設定されているか確認
- ワーキングディレクトリが存在するか確認

### ログが表示されない
- プロセスが実際に出力を生成しているか確認
- `stream`パラメータが適切か確認（Stdout/Stderr/Both）

### ポート競合
- 既存のプロセスを停止してから新しいプロセスを起動
- `lsof -i :PORT`でポート使用状況を確認

### メモリ使用量が高い
- ログの最大行数を調整（`ICHIMI_MAX_LOG_LINES`）
- 不要なプロセスを削除

## 関連リンク

- [GitHub リポジトリ](https://github.com/chronista-club/ichimi-server)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Claude Code ドキュメント](https://docs.anthropic.com/claude-code)

## 更新履歴

- 2024-XX-XX: 初版作成
- 最新版の機能と使用方法を記載

---

*このドキュメントはClaude Codeでの効率的な開発のために作成されました。*