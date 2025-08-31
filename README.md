# Ichimi Server

[English](./README.en.md) | **日本語**

Process as a Resource - プロセスをリソースとして管理

Model Context Protocol (MCP) を介した Claude Code 用の強力なプロセス管理サーバー。

![Version](https://img.shields.io/badge/version-0.1.0--beta7-blue.svg)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)
![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)

## ✨ 特徴

### コア機能
- 🚀 **プロセス管理**: あらゆるプロセスの起動、停止、監視をMCPツール経由で制御
- 📊 **リアルタイムログ**: stdout/stderr 出力のキャプチャとストリーミング
- 🔍 **ステータス監視**: プロセスの状態とメトリクスの追跡
- 🎯 **柔軟なフィルタリング**: 状態やパターンでプロセスを検索
- 💾 **永続化**: KDL形式での設定ファイル管理（`.ichimi/processes.kdl`）
- 🔄 **自動起動**: `auto_start` フラグでサーバー起動時のプロセス自動起動

### Webダッシュボード (v0.1.0-beta7〜)
- 🌐 **モダンなUI**: Alpine.js + Tablerによる洗練されたデザイン
- 📈 **リアルタイム更新**: 5秒ごとの自動更新でプロセス状態を監視
- 🔍 **検索機能**: プロセスの検索とフィルタリング
- 🌙 **ダークモード**: ライト/ダークテーマの切り替え対応
- 📱 **レスポンシブ**: モバイルからデスクトップまで対応

### MCP統合
- 🔌 **MCP準拠サーバー**: Model Context Protocolに完全準拠
- 🤖 **Claude Code対応**: Claude Codeから直接利用可能
- 🛠️ **豊富なツール**: 12種類以上のMCPツールを提供
- 📡 **Web API**: RESTful APIによる外部統合

## 🚀 インストール

### Cargoを使用（推奨）

```bash
cargo install ichimi-server
# コマンドは 'ichimi' として利用可能になります
```

### ソースからのインストール

```bash
# リポジトリをクローン
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server

# リリースビルド
cargo build --release

# バイナリは以下の場所に生成されます:
# target/release/ichimi
```

## 🔧 設定

### Claude Codeの設定

`.mcp.json` または Claude Code の設定にサーバーを追加:

```json
{
    "mcpServers": {
        "ichimi": {
            "type": "stdio",
            "command": "ichimi",
            "env": {
                "RUST_LOG": "info"
            }
        }
    }
}
```

### 接続の確認

Claude Code で以下を実行:
```
/mcp
```

"ichimi" サーバーが "connected" と表示されるはずです。

## 📚 使い方

### 利用可能なMCPツール

#### 基本ツール
| ツール | 説明 |
|--------|------|
| `echo` | テスト用にメッセージをエコーバック |
| `ping` | シンプルなヘルスチェック |
| `get_status` | サーバーステータスと稼働時間を取得 |

#### プロセス管理ツール
| ツール | 説明 |
|--------|------|
| `create_process` | 新しいプロセス設定を登録 |
| `start_process` | 登録済みプロセスを起動 |
| `stop_process` | 実行中のプロセスを正常停止 |
| `get_process_status` | 詳細なプロセスステータスを取得 |
| `get_process_output` | プロセスの stdout/stderr ログを取得 |
| `list_processes` | フィルタを使用して管理中の全プロセスを一覧表示 |
| `remove_process` | 管理からプロセスを削除 |
| `export_processes` | 全プロセスをファイルにエクスポート |
| `import_processes` | ファイルからプロセスをインポート |

### 使用例

#### Webサーバーの管理

```python
# Webサーバープロセスを登録
create_process(
    id="webserver",
    command="python",
    args=["-m", "http.server", "8000"],
    env={"PYTHONUNBUFFERED": "1"},
    cwd="./public"
)

# サーバーを起動
start_process(id="webserver")

# ログを確認
get_process_output(id="webserver", stream="Both", lines=50)

# 正常停止
stop_process(id="webserver", grace_period_ms=5000)
```

#### Node.jsアプリケーションの管理

```python
# Node.jsアプリを登録
create_process(
    id="node-app",
    command="node",
    args=["server.js"],
    env={"NODE_ENV": "production", "PORT": "3000"},
    cwd="/app"
)

# アプリを起動
start_process(id="node-app")

# ステータスを監視
get_process_status(id="node-app")
```

#### バッチプロセス管理

```python
# 実行中の全プロセスを一覧表示
list_processes(filter={"state": "Running"})

# パターンで特定のプロセスを検索
list_processes(filter={"name_pattern": "worker"})

# 全ワーカーを停止
for process in list_processes(filter={"name_pattern": "worker"}):
    stop_process(id=process["id"])
```

## 📝 永続化

### KDL設定ファイル

Ichimi Server は、プロセス設定の永続化に [KDL (Cuddly Data Language)](https://kdl.dev/) フォーマットを使用します。設定ファイルは `.ichimi/processes.kdl` に自動的に保存されます。

#### KDL設定ファイルの例

```kdl
// Ichimi Server Process Configuration
meta {
    version "1.0.0"
}

// Webサーバープロセス
process "webserver" {
    command "python"
    args "-m" "http.server" "8000"
    cwd "/path/to/public"
    auto_start #false
}

// バックグラウンドワーカー
process "worker" {
    command "/usr/local/bin/worker"
    args "--config" "worker.conf"
    cwd "/app"
    auto_start #true  // サーバー起動時に自動起動
}
```

#### 設定項目

| 項目 | 説明 | 必須 |
|------|------|------|
| `command` | 実行するコマンドのパス | ✅ |
| `args` | コマンドライン引数（複数可） | ❌ |
| `cwd` | 作業ディレクトリ | ❌ |
| `auto_start` | サーバー起動時の自動起動 | ❌ |

### JSONエクスポート/インポート

プロセス設定はJSON形式でもエクスポート/インポートできます：

```bash
# プロセスをJSONファイルにエクスポート
curl http://127.0.0.1:12700/api/export > ichimi_export.json

# JSONファイルからプロセスをインポート
curl -X POST http://127.0.0.1:12700/api/import \
  -H "Content-Type: application/json" \
  -d @ichimi_export.json
```

## 🌐 Webダッシュボード

### ダッシュボードの起動

```bash
# Webダッシュボードで起動（デフォルトポート 12700）
ichimi --web

# カスタムポートを指定
ichimi --web --web-port 8080

# Webダッシュボードのみ（MCPサーバーなし）
ichimi --web-only
```

ブラウザで `http://localhost:12700` を開きます。

### ダッシュボード機能

#### メイン画面
- **統計カード**: 総プロセス数、実行中、停止中、エラーの状態を表示
- **プロセスリスト**: 全プロセスのテーブル表示
- **リアルタイム更新**: 5秒ごとに自動更新
- **検索機能**: プロセスID、コマンドで検索

#### プロセス操作
- **起動/停止**: ワンクリックでプロセス制御
- **ログ表示**: stdout/stderrの最新ログを表示
- **削除**: 不要なプロセスを削除
- **新規追加**: モーダルダイアログでプロセス追加

#### UI/UX
- **レスポンシブデザイン**: モバイル対応
- **ダークモード**: ライト/ダークテーマ切り替え
- **モダンなデザイン**: Tabler UIフレームワーク使用

### REST API

| エンドポイント | メソッド | 説明 |
|--------------|----------|------|
| `/api/status` | GET | サーバーステータス |
| `/api/dashboard` | GET | ダッシュボード統計データ |
| `/api/processes` | GET | プロセス一覧 |
| `/api/processes` | POST | プロセス追加 |
| `/api/processes/:id` | GET | プロセス詳細 |
| `/api/processes/:id` | DELETE | プロセス削除 |
| `/api/processes/:id/start` | POST | プロセス起動 |
| `/api/processes/:id/stop` | POST | プロセス停止 |
| `/api/processes/:id/logs` | GET | ログ取得 |

## 🏗️ プロジェクト構造

```
ichimi-server/
├── src/
│   ├── lib.rs                  # コアサーバー実装
│   ├── bin/
│   │   └── ichimi_server.rs    # バイナリエントリーポイント
│   ├── process/
│   │   ├── mod.rs              # プロセスモジュールのエクスポート
│   │   ├── manager.rs          # プロセスライフサイクル管理
│   │   ├── buffer.rs           # ログ用循環バッファ
│   │   └── types.rs            # 型定義
│   ├── web/
│   │   ├── mod.rs              # Webサーバーモジュール  
│   │   ├── server.rs           # HTTPサーバー実装
│   │   ├── handlers.rs         # APIハンドラー
│   │   └── api.rs              # APIルーティング
│   ├── messages/
│   │   └── mod.rs              # MCP メッセージ型
│   ├── persistence/
│   │   ├── mod.rs              # 永続化モジュール
│   │   ├── manager.rs          # 永続化マネージャー
│   │   ├── kdl_persistence.rs  # KDL形式の永続化（メイン）
│   │   └── kdl_schema.rs       # KDLスキーマ定義
│   ├── db/                     # SurrealDBモジュール（将来拡張用、現在未使用）
│   ├── events/                 # イベントシステム（将来拡張用、現在未使用）
│   └── learning/               # 学習システム（将来拡張用、現在未使用）
├── templates/                   # Teraテンプレート
│   ├── base.tera               # ベーステンプレート
│   └── index.tera              # ダッシュボード画面
├── .ichimi/                     # データディレクトリ
│   └── processes.kdl           # プロセス設定ファイル
└── examples/                    # 使用例
```

## 🔑 環境変数

| 変数 | 説明 | デフォルト |
|------|------|----------|
| `RUST_LOG` | ログレベル (error, warn, info, debug, trace) | info |
| `ICHIMI_DATA_DIR` | データファイル用ディレクトリ | ~/.ichimi/data |
| `ICHIMI_IMPORT_FILE` | 起動時にインポートするファイル | ~/.ichimi/data/processes.surql |
| `ICHIMI_EXPORT_FILE` | シャットダウン時のエクスポート先 | ~/.ichimi/data/processes.surql |
| `ICHIMI_STOP_ON_SHUTDOWN` | ichimi終了時にプロセスを停止するか（true/false） | false（継続） |
| `ICHIMI_AUTO_EXPORT_INTERVAL` | 自動エクスポート間隔（秒） | なし |

## 🚧 開発

### ローカル開発

```bash
# デバッグビルド
cargo build

# テストの実行
cargo test

# デバッグログで実行
RUST_LOG=debug cargo run

# Webダッシュボードの開発
RUST_LOG=debug cargo run -- --web
```

### リリースビルド

```bash
# 最適化されたリリースビルド
cargo build --release

# インストール
cargo install --path .
```

## 🤝 コントリビューション

コントリビューションを歓迎します！

1. リポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを開く

## 📄 ライセンス

このプロジェクトは以下のいずれかのライセンスでデュアルライセンスされています:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) または http://opensource.org/licenses/MIT)

お好みの方をお選びください。

## 🙏 謝辞

- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- [Tera](https://tera.netlify.app/) - テンプレートエンジン
- UIフレームワーク: [Alpine.js](https://alpinejs.dev/) & [Tabler](https://tabler.io/)
- [KDL](https://kdl.dev/) - 設定フォーマット
- Model Context Protocol 仕様に触発
- Chronista Club エコシステムの一部

## 📞 サポート

問題、質問、提案については:
- [GitHub Issues](https://github.com/chronista-club/ichimi-server/issues) で Issue を開く
- [Discussions](https://github.com/chronista-club/ichimi-server/discussions) で議論

---

*Ichimi Server - Claude Code のためのシンプルかつ強力なプロセス管理。一味が支えます。*

**Latest Release:** v0.1.0-beta7 🎉