# 使用ガイド

## 基本的な使い方

### プロセスの作成と実行

1. **プロセスを作成**
```
Create a process called "web-server" that runs "python -m http.server 8000"
```

2. **プロセスを起動**
```
Start the web-server process
```

3. **プロセスの状態を確認**
```
Get status of web-server process
```

4. **プロセスの出力を確認**
```
Show output from web-server process
```

5. **プロセスを停止**
```
Stop the web-server process
```

## 高度な使用例

### 複数プロセスの管理

```
# 複数のワーカープロセスを作成
Create process "worker-1" running "python worker.py --id 1"
Create process "worker-2" running "python worker.py --id 2"
Create process "worker-3" running "python worker.py --id 3"

# すべてを起動
Start all worker processes

# 実行中のプロセスをリスト
List all running processes
```

### 環境変数とワーキングディレクトリ

```
# 環境変数とディレクトリを指定してプロセスを作成
Create process "app" with:
- Command: npm run dev
- Working directory: /home/user/myapp
- Environment: NODE_ENV=development, PORT=3000
```

### 自動再起動の設定

```
# 復元時に自動起動するプロセスを作成
Create process "database" running "postgres" with auto-start enabled
```

### CI/CD監視

```
# 最新のCI実行を確認
List recent CI runs

# 特定のCI実行の詳細を取得
Get details of CI run #12345

# CI完了を待機
Wait for CI run #12345 to complete
```

## Webダッシュボード

### 起動方法

```bash
ichimi --web
```

ブラウザで `http://localhost:12700` を開きます。

### 機能

- **プロセス一覧**: すべてのプロセスの状態を一覧表示
- **リアルタイムログ**: プロセスの出力をリアルタイムで表示
- **プロセス制御**: Web UIからプロセスの起動/停止
- **システムモニタリング**: CPU/メモリ使用率の監視

## 永続化とバックアップ

### 手動エクスポート

```
Export all processes to backup file
```

### 手動インポート

```
Import processes from backup file
```

### 自動バックアップ

環境変数で設定：
```bash
export ICHIMI_AUTO_EXPORT_INTERVAL=300  # 5分ごと
export ICHIMI_EXPORT_FILE=~/.ichimi/backup.yml
```

## ベストプラクティス

### 1. プロセスID の命名規則

- 小文字とハイフンを使用: `web-server`, `background-worker`
- 意味のある名前を使用
- 一意性を保証

### 2. リソース管理

- 長時間実行プロセスは定期的に出力を確認
- 不要なプロセスは適切に停止
- メモリ使用量を監視

### 3. エラーハンドリング

- プロセス起動前にコマンドの有効性を確認
- 失敗したプロセスのログを確認
- 適切なタイムアウトを設定

### 4. セキュリティ

- 信頼できるコマンドのみ実行
- 環境変数に機密情報を含めない
- 適切な権限でプロセスを実行

## トラブルシューティング

### プロセスが起動しない

1. コマンドが正しいか確認
2. 必要な実行ファイルがPATHに存在するか確認
3. ワーキングディレクトリが存在するか確認

### プロセスがすぐに終了する

1. プロセスの出力を確認してエラーメッセージを探す
2. 必要な環境変数が設定されているか確認
3. 依存関係が満たされているか確認

### 出力が表示されない

1. プロセスが実際に出力を生成しているか確認
2. バッファリングの問題の可能性（`-u` フラグなどを使用）
3. 出力制限に達していないか確認