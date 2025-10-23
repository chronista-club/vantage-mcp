# Vantage Server 環境変数

## ロギング
- `RUST_LOG`: ログレベル設定
  - 値: `error`, `warn`, `info`, `debug`, `trace`
  - 例: `RUST_LOG=info`
  - モジュール別設定: `RUST_LOG=info,vantage_server::persistence=debug`

## 永続化設定
- `ICHIMI_AUTO_EXPORT_INTERVAL`: 自動エクスポート間隔（秒）
  - 例: `ICHIMI_AUTO_EXPORT_INTERVAL=300` (5分ごと)
  
- `ICHIMI_IMPORT_FILE`: 起動時にインポートするファイル
  - 例: `ICHIMI_IMPORT_FILE=backup.json`

- `ICHIMI_DATA_DIR`: データファイル用ディレクトリ
  - デフォルト: `~/.vantage/data`
  - 例: `ICHIMI_DATA_DIR=/custom/path/data`

- `ICHIMI_DB_PATH`: SurrealDBのパス
  - デフォルト: `./data/vantage.db`
  - 例: `ICHIMI_DB_PATH=/var/lib/vantage/db`

## Webサーバー設定（コマンドライン引数）
- `--web`: Webダッシュボードを有効化
- `--web-only`: MCPサーバーを無効化し、Webのみ起動
- `--web-port <PORT>`: Webサーバーのポート指定（デフォルト: 12700）

## デバッグ用
- `RUST_BACKTRACE`: パニック時のバックトレース表示
  - 値: `1` または `full`
  - 例: `RUST_BACKTRACE=1 cargo run`