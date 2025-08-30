<div align="center">

# 🍡 Ichimi Server

**Process as a Resource**

*Model Context Protocol (MCP) を介した Claude Code 用の強力なプロセス管理サーバー*

[![Version](https://img.shields.io/badge/version-0.1.0--beta8-blue.svg)](https://github.com/chronista-club/ichimi-server)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](https://github.com/chronista-club/ichimi-server)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-compatible-purple.svg)](https://modelcontextprotocol.io/)

*あらゆるプロセスを Claude Code から直接管理・監視できる、シンプルで強力なツール*

</div>

## ✨ 主な特徴

### 🎯 **Claude Code との完全統合**
Model Context Protocol (MCP) を使用して、Claude Code から直接プロセスを管理できます。

### 🚀 **パワフルなプロセス管理**
- **完全なライフサイクル制御**: 起動、停止、再起動、監視
- **リアルタイムログストリーミング**: stdout/stderr のライブキャプチャ
- **インテリジェントなフィルタリング**: 名前、状態、パターンでの検索
- **グレースフル シャットダウン**: 安全なプロセス終了

### 💾 **永続化と信頼性**
- **SurrealDB 駆動**: インメモリデータベースによる高速動作
- **自動バックアップ**: 設定可能な間隔でのデータエクスポート
- **完全復旧**: いつでもプロセス設定を復元可能

### 🌐 **使いやすさ**
- **Webダッシュボード**: 直感的なブラウザUI（オプション）
- **ゼロ設定**: すぐに使い始められる
- **Rust製**: 高性能で安全

## 🚀 クイックスタート

### 1. インストール

**推奨方法: Cargo を使用**
```bash
cargo install ichimi-server
```

**ソースからビルド**
```bash
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server
cargo build --release
# バイナリ: target/release/ichimi
```

### 2. Claude Code との連携設定

`.mcp.json` に以下を追加:

```json
{
    "mcpServers": {
        "ichimi": {
            "type": "stdio",
            "command": "ichimi"
        }
    }
}
```

### 3. 動作確認

Claude Code で確認:
```
/mcp
```
「ichimi」が「connected」と表示されれば成功です！

### 4. 最初のプロセスを管理してみよう

```python
# シンプルなWebサーバーを起動
create_process(
    id="demo-server", 
    command="python", 
    args=["-m", "http.server", "8080"]
)
start_process(id="demo-server")

# ログを確認
get_process_output(id="demo-server", lines=10)
```

## ⚙️ 高度な設定

### 環境変数

| 変数名 | 説明 | デフォルト値 |
|--------|------|-------------|
| `RUST_LOG` | ログレベル (debug, info, warn, error) | `info` |
| `ICHIMI_AUTO_EXPORT_INTERVAL` | 自動バックアップ間隔（秒） | 無効 |
| `ICHIMI_IMPORT_FILE` | 起動時インポートファイル | なし |
| `ICHIMI_DATA_DIR` | データディレクトリ | `~/.ichimi/data` |

### Claude Code 設定例

```json
{
    "mcpServers": {
        "ichimi": {
            "type": "stdio",
            "command": "ichimi",
            "env": {
                "RUST_LOG": "info",
                "ICHIMI_AUTO_EXPORT_INTERVAL": "300"
            }
        }
    }
}
```

## 💡 実用的なユースケース

### 🌐 Web 開発

**開発サーバーの管理**
```python
# フロントエンド開発サーバー
create_process(
    id="vite-dev",
    command="npm", 
    args=["run", "dev"],
    cwd="./frontend"
)

# バックエンドAPI
create_process(
    id="api-server",
    command="cargo",
    args=["run", "--bin", "api"],
    env={"RUST_LOG": "debug"}
)

# 同時起動
start_process(id="vite-dev")
start_process(id="api-server")
```

### 🗄️ データベース運用

**複数DBの管理**
```python
# PostgreSQL
create_process(
    id="postgres",
    command="postgres",
    args=["-D", "/usr/local/var/postgres"]
)

# Redis
create_process(
    id="redis",
    command="redis-server",
    args=["--port", "6379"]
)
```

### 🔄 CI/CD とタスク管理

**ビルドパイプラインの監視**
```python
# テストの実行
create_process(
    id="test-suite",
    command="cargo",
    args=["test", "--", "--nocapture"]
)

# リアルタイムでログを監視
get_process_output(id="test-suite", stream="Both")
```

## 📋 利用可能なツール

### 基本操作
- `echo` - メッセージのエコーテスト
- `ping` - ヘルスチェック  
- `get_status` - サーバー稼働状況

### プロセス管理
- `create_process` - プロセス設定の登録
- `start_process` - プロセスの起動
- `stop_process` - プロセスの停止（グレースフル）
- `get_process_status` - 詳細ステータス取得
- `get_process_output` - ログ出力の取得
- `list_processes` - プロセス一覧（フィルタ対応）
- `remove_process` - プロセス設定の削除

### データ管理
- `export_processes` - 設定の `.surql` ファイルエクスポート
- `import_processes` - 設定の `.surql` ファイルインポート

## 🌐 Webダッシュボード

Ichimi Server には美しく直感的な Web UI が付属しています。

### 起動方法

```bash
# デフォルトポート（12700）で起動
ichimi --web

# カスタムポート指定
ichimi --web --web-port 8080
```

ブラウザで `http://localhost:12700` を開くとダッシュボードにアクセスできます。

### 機能
- 📊 **リアルタイム監視**: プロセス状態の即座な更新
- 🎛️ **ワンクリック操作**: 起動・停止・再起動が簡単
- 📝 **ライブログ**: stdout/stderr の自動更新表示
- 🔍 **高度なフィルタ**: 状態・名前での絞り込み
- 🎨 **Tabler UI**: モダンでレスポンシブなデザイン

## 📖 API リファレンス

### プロセス状態
- `NotStarted` - 登録済み・未起動
- `Running` - 実行中（PID あり）  
- `Stopped` - 正常終了
- `Failed` - 異常終了

### 出力ストリーム
- `Stdout` - 標準出力のみ
- `Stderr` - エラー出力のみ
- `Both` - 両方を結合

## 💾 データの永続化

### SurrealDB による高速ストレージ

Ichimi は**インメモリ SurrealDB** を使用してプロセス設定を管理し、高速な読み書きを実現しています。

### 自動バックアップ

```bash
# 5分間隔での自動エクスポート
ICHIMI_AUTO_EXPORT_INTERVAL=300 ichimi

# 起動時の自動復旧
ICHIMI_IMPORT_FILE=/path/to/backup.surql ichimi
```

### 手動操作

```python
# バックアップ作成
export_processes(file_path="/path/to/backup.surql")

# 復旧
import_processes(file_path="/path/to/backup.surql")
```

デフォルト保存場所: `~/.ichimi/data/ichimi_export.surql`

## 🛠️ 開発者向け情報

### ビルドと実行

```bash
# 開発ビルド
cargo build

# リリースビルド  
cargo build --release

# テスト実行
cargo test

# デバッグモードで実行
RUST_LOG=debug cargo run
```

### アーキテクチャ

```
ichimi-server/
├── src/
│   ├── lib.rs                    # コアサーバー
│   ├── bin/ichimi_server.rs      # エントリーポイント
│   ├── process/                  # プロセス管理
│   │   ├── manager.rs            # ライフサイクル管理
│   │   ├── buffer.rs             # ログバッファ
│   │   └── types.rs              # 型定義
│   ├── web/                      # Webダッシュボード
│   ├── messages/                 # MCP メッセージ
│   └── persistence.rs            # SurrealDB 層
├── static/                       # Web UI アセット
└── examples/                     # サンプルコード
```

## 🤝 コントリビューション

プロジェクトへの貢献を歓迎しています！

### 貢献の流れ

1. **Fork** - リポジトリをフォーク
2. **Branch** - フィーチャーブランチを作成  
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Commit** - 変更をコミット
   ```bash
   git commit -m 'feat: Add amazing feature'
   ```
4. **Push** - ブランチにプッシュ
   ```bash
   git push origin feature/amazing-feature
   ```
5. **PR** - プルリクエストを作成

### 開発ガイドライン

- **テストを書く**: 新機能にはテストを追加
- **ドキュメント更新**: APIが変わった場合は文書も更新
- **コードフォーマット**: `cargo fmt` でフォーマット
- **Lint チェック**: `cargo clippy` でリントを通す

## 📄 ライセンス

このプロジェクトはデュアルライセンスです：

- **MIT License** - [LICENSE-MIT](LICENSE-MIT)  
- **Apache 2.0** - [LICENSE-APACHE](LICENSE-APACHE)

お好きな方を選択してください。

## 🙏 謝辞

特別な感謝を：

- **[Model Context Protocol](https://modelcontextprotocol.io/)** - 革新的な統合仕様
- **[rmcp](https://github.com/modelcontextprotocol/rust-sdk)** - Rust MCP SDK
- **[SurrealDB](https://surrealdb.com/)** - 高性能インメモリデータベース  
- **UI フレームワーク**: [Alpine.js](https://alpinejs.dev/) & [Tabler](https://tabler.io/)
- **Chronista Club エコシステム** - 革新的ツールチェーンの一部

## 💬 サポート & コミュニティ

### 問題報告・質問

- **GitHub Issues** - [バグ報告・機能要望](https://github.com/chronista-club/ichimi-server/issues)
- **Documentation** - [Wiki](https://github.com/chronista-club/ichimi-server/wiki) で詳細ガイド

### つながりましょう

- **Twitter**: [@chronistaclub](https://twitter.com/chronistaclub)
- **Chronista Club** - [他のツールも確認](https://github.com/chronista-club)

---

<div align="center">

**🍡 Ichimi Server**

*Claude Code のための、シンプルかつ強力なプロセス管理*  
*一味が支える、次世代の開発体験*

[⭐ Star on GitHub](https://github.com/chronista-club/ichimi-server) | [📖 Documentation](https://github.com/chronista-club/ichimi-server/wiki) | [🚀 Get Started](#-クイックスタート)

</div>
