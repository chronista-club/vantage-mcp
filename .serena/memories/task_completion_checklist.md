# タスク完了時のチェックリスト

## コード変更後に必ず実行すること

### 1. コードフォーマット
```bash
cargo fmt
```

### 2. Lintチェック
```bash
cargo clippy
```
警告が出た場合は可能な限り修正する

### 3. コンパイルチェック
```bash
cargo check
cargo build
```

### 4. テスト実行
```bash
cargo test
```
関連するテストがある場合は個別に実行：
```bash
cargo test test_persistence  # 永続化関連
cargo test test_process_manager  # プロセス管理関連
```

### 5. 動作確認（該当する場合）
```bash
# サーバー起動して動作確認
RUST_LOG=info cargo run --bin ichimi -- --web-only

# APIエンドポイントのテスト（curlなど）
curl http://localhost:12701/api/processes
```

### 6. Git操作
```bash
git add .
git status  # 変更内容を確認
git diff --cached  # ステージングされた変更を確認
git commit -m "適切なコミットメッセージ"
```

## 重要な変更の場合

### ドキュメント更新
- README.md / README.ja.md の更新が必要か確認
- CHANGELOG.md への記載が必要か確認
- コード内のコメント・ドキュメントが最新か確認

### 破壊的変更の確認
- APIの後方互換性が保たれているか
- 既存の設定ファイルが引き続き動作するか
- データベーススキーマの変更がある場合、マイグレーションが必要か

## CI/CD関連
- GitHub Actionsでのビルドが通るか（プッシュ後に確認）
- リリースノートの準備（バージョンアップの場合）