# 自動起動仕様 - 仕様書

## 目的

Vantage MCP サーバー起動時に、**指定されたプロセスを自動的に起動**することで、開発環境のセットアップを迅速化する。

### 解決する課題

- **手動起動の手間**: 毎回複数のプロセスを手動で起動するのは非効率
- **起動漏れ**: 必要なプロセスを起動し忘れるリスク
- **起動順序**: 依存関係のあるプロセスを正しい順序で起動する必要がある

## ユースケース

### UC-001: 開発環境の自動セットアップ

**Actor**: 開発者

**Goal**: Vantage 起動と同時に、フロントエンド、バックエンド、DB を自動起動したい

**前提**:
- プロセス "frontend-dev" の `auto_start_on_restore` = true
- プロセス "backend-api" の `auto_start_on_restore` = true
- プロセス "postgres-db" の `auto_start_on_restore` = true

**Flow**:
1. Vantage MCP サーバーを起動（`vantagemcp`）
2. スナップショットからプロセス設定を復元
3. `auto_start_on_restore = true` のプロセスが自動的に起動
4. ブラウザで WebUI を開くと、既に 3 つのプロセスが Running

**Success Criteria**: コマンド 1 つで全ての開発環境が立ち上がる

### UC-002: 選択的な自動起動

**Actor**: 開発者

**Goal**: 常に必要なプロセスのみ自動起動し、必要に応じて手動起動したい

**前提**:
- プロセス "db" の `auto_start_on_restore` = true（常に必要）
- プロセス "frontend" の `auto_start_on_restore` = false（フロントエンド開発時のみ必要）

**Flow**:
1. Vantage 起動
2. "db" のみが自動起動
3. フロントエンド開発時は、WebUI で "frontend" を手動起動

**Success Criteria**: 必要最小限のプロセスのみが起動し、リソースを節約

## 機能要件

### FR-001: auto_start_on_restore フラグ（必須）

- **説明**: プロセスごとに自動起動の可否を設定
- **型**: `bool`
- **デフォルト**: `false`
- **設定方法**:
  - プロセス作成時に指定
  - プロセス編集で変更可能

### FR-002: 起動時の自動起動処理（必須）

- **トリガー**: Vantage MCP サーバー起動時
- **処理フロー**:
  1. スナップショット（`.vantage/snapshot.yaml` または SurrealDB）からプロセス設定を復元
  2. `auto_start_on_restore = true` のプロセスを抽出
  3. 依存関係順（将来実装）または登録順に起動
  4. 起動失敗時はログに記録し、次のプロセスに進む

### FR-003: 起動失敗時のハンドリング（必須）

- **説明**: 自動起動に失敗した場合の挙動
- **処理**:
  - エラーログを記録
  - プロセス状態を Failed に設定
  - 他のプロセスの起動は継続
  - WebUI でエラーを通知

### FR-004: 起動順序の制御（オプション、将来実装）

- **説明**: 依存関係に基づいて起動順序を制御
- **例**: DB → バックエンド → フロントエンドの順に起動
- **実装方針**: プロセスに `depends_on` フィールドを追加

### FR-005: 起動遅延の設定（オプション、将来実装）

- **説明**: プロセス起動後、次のプロセス起動まで待機
- **例**: DB 起動後 5 秒待ってからバックエンド起動
- **実装方針**: プロセスに `startup_delay_ms` フィールドを追加

## 非機能要件

### NFR-001: 起動時間

- **目標**: 5 つのプロセスを 10 秒以内に起動
- **測定**: 各プロセスの起動時間をログに記録

### NFR-002: 失敗時の復旧

- **目標**: 1 つのプロセス起動失敗が他のプロセスに影響しない
- **保証**: 各プロセスの起動は独立して処理

### NFR-003: ログの可視性

- **目標**: 自動起動のログを確認可能
- **実装**: stdout に起動ログを出力

## 設計

### データモデル

```rust
// crates/vantage-atom/src/messages/process.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub state: ProcessState,
    pub auto_start_on_restore: bool,  // ← 自動起動フラグ
    pub created_at: String,
}
```

### 起動シーケンス

```rust
// crates/vantage-atom/src/lib.rs
impl VantageServer {
    pub async fn initialize(&mut self) -> Result<(), String> {
        // 1. スナップショットを復元
        self.restore_snapshot().await?;

        // 2. 自動起動対象のプロセスを抽出
        let auto_start_processes = self.process_manager
            .list_processes()
            .await
            .into_iter()
            .filter(|p| p.auto_start_on_restore && matches!(p.state, ProcessState::NotStarted))
            .collect::<Vec<_>>();

        // 3. 順次起動
        for process in auto_start_processes {
            match self.process_manager.start_process(&process.id).await {
                Ok(_) => {
                    tracing::info!("Auto-started process: {}", process.id);
                }
                Err(e) => {
                    tracing::error!("Failed to auto-start process {}: {}", process.id, e);
                    // エラーを記録するが、次のプロセスに進む
                }
            }
        }

        Ok(())
    }
}
```

### 起動ログ例

```
2025-11-02T10:00:00Z INFO  Vantage MCP Server starting...
2025-11-02T10:00:00Z INFO  Restoring snapshot from ~/.vantage/snapshot.yaml
2025-11-02T10:00:00Z INFO  Restored 5 processes
2025-11-02T10:00:01Z INFO  Auto-starting process: postgres-db
2025-11-02T10:00:02Z INFO  Auto-started process: postgres-db (PID: 12345)
2025-11-02T10:00:02Z INFO  Auto-starting process: backend-api
2025-11-02T10:00:03Z INFO  Auto-started process: backend-api (PID: 12346)
2025-11-02T10:00:03Z INFO  Auto-starting process: frontend-dev
2025-11-02T10:00:04Z ERROR Failed to auto-start process frontend-dev: Command not found: npm
2025-11-02T10:00:04Z INFO  Auto-start completed (2/3 successful)
```

## 設定方法

### プロセス作成時に指定

**MCP ツール**:
```json
{
  "tool": "create_process",
  "arguments": {
    "id": "my-app",
    "command": "npm",
    "args": ["start"],
    "auto_start_on_restore": true  // ← 自動起動を有効化
  }
}
```

**WebUI**:
- プロセス作成フォームに「自動起動」チェックボックスを追加

### プロセス編集で変更

**MCP ツール**:
```json
{
  "tool": "update_process_config",
  "arguments": {
    "id": "my-app",
    "auto_start_on_restore": false  // ← 自動起動を無効化
  }
}
```

**WebUI**:
- プロセス詳細ページに「自動起動」トグルボタンを追加

## テストケース

### TC-001: 自動起動の成功

**前提**:
- プロセス "app1" の `auto_start_on_restore` = true
- プロセス "app2" の `auto_start_on_restore` = false

**操作**: Vantage を起動

**期待結果**:
- "app1" が自動起動され、状態が Running
- "app2" は NotStarted のまま

### TC-002: 自動起動の失敗

**前提**:
- プロセス "broken-app" の `auto_start_on_restore` = true
- コマンドが不正（存在しないコマンド）

**操作**: Vantage を起動

**期待結果**:
- "broken-app" の起動が失敗
- 状態が Failed
- エラーログが記録される
- 他のプロセスの起動は継続

### TC-003: 複数プロセスの自動起動

**前提**:
- プロセス "db", "api", "web" の全てが `auto_start_on_restore` = true

**操作**: Vantage を起動

**期待結果**:
- 3 つのプロセスが全て Running
- 起動順序は登録順（将来的には依存関係順）

### TC-004: auto_start_on_restore の変更

**操作**:
1. プロセス "app" の `auto_start_on_restore` を false → true に変更
2. Vantage を再起動

**期待結果**:
- 再起動後、"app" が自動起動される

## 環境変数

| 変数 | 説明 | デフォルト |
|------|------|------------|
| `VANTAGE_AUTO_START_ENABLED` | 自動起動機能の有効/無効 | true |
| `VANTAGE_AUTO_START_DELAY_MS` | 各プロセス起動間の待機時間（ms） | 0 |

**使用例**:
```bash
# 自動起動を無効化
VANTAGE_AUTO_START_ENABLED=false vantagemcp

# 各プロセス起動間に 2 秒待機
VANTAGE_AUTO_START_DELAY_MS=2000 vantagemcp
```

## 制約

- **起動順序**: 現在は登録順に起動（依存関係は未実装）
- **並列起動**: 現在は順次起動（将来的に並列化を検討）
- **リトライ**: 起動失敗時の自動リトライは未実装

## 今後の拡張

### 依存関係の定義

```yaml
processes:
  - id: postgres-db
    auto_start_on_restore: true
    depends_on: []  # 依存なし

  - id: backend-api
    auto_start_on_restore: true
    depends_on:
      - postgres-db  # DB が起動してから起動

  - id: frontend-dev
    auto_start_on_restore: true
    depends_on:
      - backend-api  # API が起動してから起動
```

### ヘルスチェック

起動後、プロセスが正常に動作しているか確認：
```yaml
processes:
  - id: backend-api
    health_check:
      type: http
      url: http://localhost:3000/health
      interval: 5s
      timeout: 2s
      retries: 3
```

### 起動グループ

複数のプロセスをグループ化し、グループ単位で自動起動：
```yaml
groups:
  - name: development
    processes:
      - postgres-db
      - backend-api
      - frontend-dev
```

## 更新履歴

| 日付 | 変更者 | 変更内容 | 理由 |
|------|--------|----------|------|
| 2025-11-02 | Claude Code | 初版作成 | 仕様の明確化 |
