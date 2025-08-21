# RocksDB ローカル環境セットアップガイド

## 概要
ローカル開発環境でRocksDBを使用してデータを永続化する設定方法です。

## 現在の構成

### Kubernetes環境（既存）
- SurrealDB → RocksDB (`rocksdb:/data/database`)
- データはPersistentVolume内に保存

### ローカル環境への移行案

## 方法1: SurrealDBをローカルで起動（推奨）

```bash
# データディレクトリの作成
export DB_DIR="$HOME/.diarkis/data"
mkdir -p "$DB_DIR"

# SurrealDBをRocksDBバックエンドで起動
surreal start \
  --log info \
  --user diarkis \
  --pass diarkis \
  rocksdb://"$DB_DIR"/main.db
```

### Ichimiサーバーでの管理

```typescript
// Ichimiでプロセスを作成
await mcp__ichimi-server__create_process({
  id: "surrealdb-local",
  command: "surreal",
  args: [
    "start",
    "--log", "info",
    "--user", "diarkis",
    "--pass", "diarkis",
    `rocksdb://${process.env.HOME}/.diarkis/data/main.db`
  ],
  env: {
    "SURREAL_NAMESPACE": "diarkis",
    "SURREAL_DATABASE": "main"
  }
})

// プロセスを起動
await mcp__ichimi-server__start_process({
  id: "surrealdb-local"
})
```

## 方法2: Dockerコンテナで起動

```yaml
# docker-compose.yml
version: '3.8'
services:
  surrealdb:
    image: surrealdb/surrealdb:v2.3.7
    command: start --log info --user diarkis --pass diarkis rocksdb://data/database
    volumes:
      - ${DB_DIR:-~/.diarkis/data}:/data
    ports:
      - "30800:8000"
    environment:
      - SURREAL_NAMESPACE=diarkis
      - SURREAL_DATABASE=main
```

## 方法3: mise（旧rtx）でのタスク設定

`.mise.toml`に追加：

```toml
[tasks.db:local]
description = "ローカルSurrealDBをRocksDBバックエンドで起動"
run = """
  export DB_DIR="${DB_DIR:-$HOME/.diarkis/data}"
  mkdir -p "$DB_DIR"
  surreal start \
    --log info \
    --user diarkis \
    --pass diarkis \
    --bind 0.0.0.0:30800 \
    rocksdb://"$DB_DIR"/main.db
"""

[tasks.db:stop]
description = "ローカルSurrealDBを停止"
run = "pkill -f 'surreal start' || true"

[tasks.db:clean]
description = "ローカルデータベースをクリーンアップ"
run = """
  read -p "本当にデータベースを削除しますか？ (y/N): " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "${DB_DIR:-$HOME/.diarkis/data}"
    echo "データベースを削除しました"
  fi
"""
```

## アプリケーション側の設定

### 環境変数の設定

```bash
# ローカルSurrealDB用
export SURREAL_URL="ws://localhost:30800/rpc"
export SURREAL_NS="diarkis"
export SURREAL_DB="main"
export SURREAL_USERNAME="diarkis"
export SURREAL_PASSWORD="diarkis"

# データディレクトリ
export DB_DIR="$HOME/.diarkis/data"
```

### .envファイル

```env
# .env.local
SURREAL_URL=ws://localhost:30800/rpc
SURREAL_NS=diarkis
SURREAL_DB=main
SURREAL_USERNAME=diarkis
SURREAL_PASSWORD=diarkis
DB_DIR=${HOME}/.diarkis/data
```

## データのバックアップとリストア

### バックアップ

```bash
# SurrealDBのエクスポート
surreal export \
  --conn ws://localhost:30800 \
  --user diarkis \
  --pass diarkis \
  --ns diarkis \
  --db main \
  backup.surql
```

### リストア

```bash
# SurrealDBへのインポート
surreal import \
  --conn ws://localhost:30800 \
  --user diarkis \
  --pass diarkis \
  --ns diarkis \
  --db main \
  backup.surql
```

## RocksDBの直接操作（上級者向け）

```bash
# RocksDBツールのインストール
brew install rocksdb

# データベースの統計情報
ldb --db="$DB_DIR/main.db" stats

# キーの一覧
ldb --db="$DB_DIR/main.db" scan

# 特定のキーの値を取得
ldb --db="$DB_DIR/main.db" get <key>
```

## トラブルシューティング

### ポート競合
```bash
# ポート30800が使用中か確認
lsof -i :30800

# プロセスを終了
kill -9 <PID>
```

### 権限エラー
```bash
# データディレクトリの権限を修正
chmod -R 755 "$DB_DIR"
```

### データベース破損
```bash
# RocksDBの修復
ldb --db="$DB_DIR/main.db" repair
```

## ベストプラクティス

1. **定期的なバックアップ**
   - cronやsystemdタイマーで自動化
   - 開発作業前後でのスナップショット

2. **データディレクトリの管理**
   - `$HOME/.diarkis/data`に統一
   - .gitignoreに追加

3. **パフォーマンスチューニング**
   - RocksDBのキャッシュサイズ調整
   - 圧縮設定の最適化

4. **セキュリティ**
   - 本番環境では強力なパスワードを使用
   - ファイアウォール設定
   - TLS/SSL接続の有効化