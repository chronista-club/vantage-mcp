# Ichimi Server アーキテクチャ

## 概要

Ichimi Server は Model Context Protocol (MCP) を介して Claude Code と連携するプロセス管理サーバーです。

## システム構成

```
ichimi-server/
├── crates/
│   ├── ichimi/              # メインサーバークレート
│   │   ├── src/
│   │   │   ├── lib.rs       # MCP ツールハンドラー
│   │   │   ├── process/     # プロセス管理
│   │   │   ├── web/         # Webダッシュボード
│   │   │   ├── ci/          # CI/CD監視
│   │   │   ├── events/      # イベントシステム
│   │   │   ├── learning/    # 学習エンジン
│   │   │   └── security/    # セキュリティ検証
│   │   └── tests/           # 統合テスト
│   └── ichimi-persistence/  # 永続化レイヤー
│       └── src/
│           ├── persistence/  # インメモリストレージ
│           ├── kdl/         # KDL形式の設定管理
│           └── yaml/        # YAMLスナップショット
└── ui/
    └── web/                 # Vue.js Webダッシュボード
```

## 主要コンポーネント

### 1. プロセス管理 (`ProcessManager`)

プロセスのライフサイクル全体を管理します。

**特徴:**
- 非同期プロセス実行
- リアルタイム出力キャプチャ
- グレースフルシャットダウン
- 自動再起動ポリシー

**状態遷移:**
```
NotStarted → Running → Stopped/Failed
```

### 2. 永続化層 (`PersistenceManager`)

v0.2.0より、SurrealDBからインメモリストレージに移行しました。

**実装:**
- `Arc<RwLock<HashMap>>` によるインメモリストレージ
- KDL形式での設定ファイル管理（`.ichimi/processes.kdl`）
- YAMLスナップショット機能

### 3. MCP統合

Model Context Protocol を通じて以下のツールを提供：

**基本ツール:**
- `echo`, `ping`, `get_status`

**プロセス管理:**
- `create_process`, `start_process`, `stop_process`
- `get_process_status`, `get_process_output`
- `list_processes`, `remove_process`

**CI/CD監視:**
- `list_ci_runs`, `get_ci_run_details`
- `wait_for_ci_completion`

### 4. Webダッシュボード

Vue 3 + TypeScript で構築されたモダンなWeb UI。

**技術スタック:**
- Vue 3 + Composition API
- TypeScript
- Vite
- Tabler UI Framework
- Pinia (状態管理)

## セキュリティ

### 入力検証
- コマンドインジェクション対策
- パストラバーサル防止
- 環境変数サニタイゼーション

### プロセス分離
- 各プロセスは独立したコンテキストで実行
- リソース制限の適用可能

## パフォーマンス最適化

### メモリ管理
- `CircularBuffer` による固定容量ログ管理
- 長時間実行プロセスのメモリ効率化

### 並行処理
- `Arc<RwLock>` パターンによるスレッドセーフな並行アクセス
- 細粒度ロックによる競合の最小化

## 設計原則

1. **シンプリシティ**: 外部依存を最小限に
2. **拡張性**: モジュラー設計による機能追加の容易さ
3. **信頼性**: エラーハンドリングとグレースフルリカバリ
4. **パフォーマンス**: 効率的なリソース利用