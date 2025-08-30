# Ichimi Server コードベース構造

## ディレクトリ構造
```
ichimi-server/
├── src/
│   ├── lib.rs              # メインサーバー実装、MCPツールハンドラー
│   ├── bin/
│   │   └── ichimi_server.rs # エントリーポイント
│   ├── messages/           # リクエスト/レスポンスメッセージ
│   │   ├── basic.rs       # 基本メッセージタイプ
│   │   ├── process.rs     # プロセス管理リクエスト
│   │   └── suggestions.rs # 提案関連メッセージ
│   ├── process/            # プロセス管理ロジック
│   │   ├── manager.rs     # ProcessManager実装
│   │   ├── buffer.rs      # CircularBuffer（ログ保存）
│   │   └── types.rs       # ドメインタイプ定義
│   ├── persistence/        # 永続化層
│   │   ├── manager.rs     # PersistenceManager（SurrealDB）
│   │   ├── kdl_persistence.rs # KDLファイル操作
│   │   └── kdl_schema.rs  # KDLスキーマ定義
│   ├── db/                # データベース層
│   │   └── mod.rs         # SurrealDBクライアント
│   ├── events/            # イベントシステム
│   ├── learning/          # 学習エンジン
│   └── web/               # Webダッシュボード
│       ├── server.rs      # HTTPサーバー
│       ├── api.rs         # REST API
│       └── handlers.rs    # リクエストハンドラー
├── tests/                 # 統合テスト
├── examples/              # サンプルコード
├── static/                # Web静的ファイル
├── data/                  # データベースファイル
│   ├── ichimi.db/        # SurrealDB RocksDB
│   └── setup.surql       # DBスキーマ定義
├── .claude/              # Claude Code設定
│   └── CLAUDE.md         # プロジェクト固有の指示
└── .serena/              # Serena設定

## 主要な設計パターン

1. **Arc<RwLock> パターン**: スレッドセーフな並行アクセス
2. **ステートマシン**: プロセス状態遷移（NotStarted → Running → Stopped/Failed）
3. **非同期処理**: tokioベースの非同期I/O
4. **MCPツールルーター**: `#[tool]`属性でツールを定義
5. **永続化アーキテクチャ**: SurrealDBで永続化、KDLでインポート/エクスポート
```