# Ichimi Server

Model Context Protocol (MCP) を介した Claude Code 用の強力なプロセス管理サーバー。

## 特徴

- 🚀 **プロセス管理**: あらゆるプロセスの起動、停止、監視
- 📊 **リアルタイムログ**: stdout/stderr 出力のキャプチャとストリーミング
- 🔍 **ステータス監視**: プロセスの状態とメトリクスの追跡
- 🎯 **柔軟なフィルタリング**: フィルタを使用したプロセスの一覧表示と検索
- 💾 **メモリ効率**: ログ管理用の循環バッファ
- 🔌 **MCP ネイティブ**: Claude Code 統合に特化して構築

## インストール

### ソースからのインストール

```bash
# リポジトリをクローン
git clone https://github.com/chronista-club/ichimi-server
cd ichimi-server

# サーバーをビルド
cargo build --release

# バイナリは以下の場所に生成されます:
# target/release/ichimi
```

### Cargo を使用

```bash
cargo install ichimi-server
# コマンドは 'ichimi' として利用可能になります
```

## 設定

### Claude Code の設定

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

## 使い方

### 利用可能なツール

#### 基本ツール
- `echo` - テスト用にメッセージをエコーバック
- `ping` - シンプルなヘルスチェック
- `get_status` - サーバーステータスと稼働時間を取得

#### プロセス管理
- `create_process` - 新しいプロセス設定を登録
- `start_process` - 登録済みプロセスを起動
- `stop_process` - 実行中のプロセスを正常停止
- `get_process_status` - 詳細なプロセスステータスを取得
- `get_process_output` - プロセスの stdout/stderr ログを取得
- `list_processes` - フィルタを使用して管理中の全プロセスを一覧表示
- `remove_process` - 管理からプロセスを削除

### 使用例

#### Web サーバーの管理

```python
# Web サーバープロセスを登録
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

#### データベースの実行

```python
# PostgreSQL を起動
create_process(
    id="postgres",
    command="postgres",
    args=["-D", "/usr/local/var/postgres"],
    env={"PGDATA": "/usr/local/var/postgres"}
)

start_process(id="postgres")

# ステータスを監視
get_process_status(id="postgres")
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

## API リファレンス

### プロセスの状態

- `NotStarted` - プロセスは登録済みだが未起動
- `Running` - プロセスは PID を持って実行中
- `Stopped` - プロセスは終了コードで正常終了
- `Failed` - プロセスはエラーメッセージで失敗

### 出力ストリーム

- `Stdout` - 標準出力のみ
- `Stderr` - 標準エラー出力のみ
- `Both` - stdout と stderr の結合

### プロセスフィルタ

- `state` - プロセス状態でフィルタ (Running/Stopped/Failed/All)
- `name_pattern` - ID パターンでフィルタ (ワイルドカード対応)

## 開発

### ソースからのビルド

```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release

# テストの実行
cargo test

# デバッグログで実行
RUST_LOG=debug cargo run
```

### プロジェクト構造

```
ichimi-server/
├── src/
│   ├── lib.rs           # コアサーバー実装
│   ├── bin/
│   │   └── ichimi_server.rs # バイナリエントリーポイント
│   └── process/
│       ├── mod.rs       # プロセスモジュールのエクスポート
│       ├── manager.rs   # プロセスライフサイクル管理
│       ├── buffer.rs    # ログ用循環バッファ
│       └── types.rs     # 型定義
├── examples/            # 使用例
└── tests/              # 統合テスト
```

## コントリビューション

コントリビューションを歓迎します！プルリクエストをお気軽に送信してください。

1. リポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add some amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを開く

## ライセンス

このプロジェクトは以下のいずれかのライセンスでデュアルライセンスされています:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) または http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) または http://opensource.org/licenses/MIT)

お好みの方をお選びください。

## 謝辞

- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK で構築
- Model Context Protocol 仕様に触発
- Chronista Club エコシステムの一部

## サポート

問題、質問、提案については:
- [GitHub](https://github.com/chronista-club/ichimi-server/issues) で Issue を開く
- [ドキュメント](https://github.com/chronista-club/ichimi-server/wiki) を確認

---

*Ichimi Server - Claude Code のためのシンプルかつ強力なプロセス管理*