# インストールガイド

## 必要条件

- Rust 1.75以降
- Git
- Claude Code (MCP対応)

## インストール方法

### 方法1: Cargoを使用（推奨）

最新リリースをインストール：
```bash
cargo install --git https://github.com/chronista-club/vantage-mcp --tag v0.2.0
```

または最新の開発版：
```bash
cargo install --git https://github.com/chronista-club/vantage-mcp
```

### 方法2: ソースからビルド

```bash
# リポジトリをクローン
git clone https://github.com/chronista-club/vantage-mcp.git
cd vantage-mcp

# リリースビルド
cargo build --release

# インストール
cargo install --path crates/vantage-atom
```

### 方法3: ローカルインストールスクリプト

```bash
# インストールスクリプトを使用
./install-local.sh
```

## Claude Code との連携設定

### 1. MCP設定ファイルを作成

`~/.config/claude/mcp.json` を作成：

```json
{
  "mcpServers": {
    "vantage": {
      "command": "vantage",
      "args": [],
      "env": {}
    }
  }
}
```

### 2. 環境変数の設定（オプション）

```bash
# ログレベルの設定
export RUST_LOG=info

# 自動エクスポート間隔（秒）
export VANTAGE_AUTO_EXPORT_INTERVAL=300

# データディレクトリ
export VANTAGE_DATA_DIR=~/.vantage/data
```

## 初回起動

```bash
# サーバーを起動
vantage

# Webダッシュボード付きで起動
vantage --web

# カスタムポートで起動
vantage --web --web-port 8080
```

## 動作確認

Claude Code で以下のコマンドを実行：

1. サーバーステータスの確認：
   - "Check Vantage server status"

2. テストプロセスの作成：
   - "Create a test process that echoes hello world"

3. プロセスの起動：
   - "Start the test process"

## トラブルシューティング

### サーバーが起動しない

1. Rustのバージョンを確認：
```bash
rustc --version
```

2. 依存関係を更新：
```bash
cargo update
```

### Claude Code が認識しない

1. MCP設定ファイルのパスを確認
2. `vantage` コマンドがPATHに含まれているか確認：
```bash
which vantage
```

3. Claude Code を再起動

### ポートが使用中

別のポートを指定：
```bash
vantage --web --web-port 12701
```

## アンインストール

```bash
cargo uninstall vantage-mcp
```

設定ファイルとデータを削除：
```bash
rm -rf ~/.vantage
rm ~/.config/claude/mcp.json  # 他のMCPサーバーがある場合は編集
```
