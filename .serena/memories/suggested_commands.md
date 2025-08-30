# Ichimi Server 開発コマンド集

## ビルドコマンド
```bash
cargo build           # デバッグビルド
cargo build --release # リリースビルド（最適化済み）
```

## テストコマンド
```bash
cargo test                    # 全テストを実行
cargo test [test_name]        # 特定のテストを実行
cargo test test_persistence   # 永続化テストを実行
```

## コード品質
```bash
cargo fmt            # コードをフォーマット
cargo fmt -- --check # フォーマットをチェック（変更なし）
cargo clippy         # リンターを実行
cargo clippy -- -D warnings # 警告をエラーとして扱う
```

## サーバーの実行
```bash
# 基本実行
cargo run --bin ichimi
./target/release/ichimi # リリースビルドを実行

# Webダッシュボード付きで実行
cargo run --bin ichimi -- --web
cargo run --bin ichimi -- --web --web-port 8080  # カスタムポート
cargo run --bin ichimi -- --web-only  # MCPサーバーなし、Webのみ

# 環境変数を設定して実行
RUST_LOG=debug cargo run
RUST_LOG=info,ichimi_server::persistence=debug cargo run
ICHIMI_AUTO_EXPORT_INTERVAL=300 cargo run  # 5分ごとに自動エクスポート
```

## Git関連
```bash
git add .
git commit -m "メッセージ"
git push
git status
git diff
```

## システムユーティリティ (Darwin/macOS)
```bash
ls -la        # ファイル一覧（隠しファイル含む）
pwd           # 現在のディレクトリ
cd [path]     # ディレクトリ移動
grep -r "pattern" .  # 再帰的に検索
find . -name "*.rs"  # ファイル検索
open .        # Finderで現在のディレクトリを開く（macOS特有）
```

## SurrealDB操作
```bash
# データベースに直接接続
surreal sql --endpoint rocksdb://data/ichimi.db --pretty

# スキーマをインポート
surreal import --namespace ichimi --database main --endpoint rocksdb://data/ichimi.db data/setup.surql
```