# 終了時仕様 - 仕様書

## 目的

Vantage MCP サーバー終了時の**プロセス処理とデータ永続化**の挙動を明確に定義し、データ損失を防ぎ、適切なクリーンアップを保証する。

### 解決する課題

- **データ損失**: サーバー終了時にプロセス設定が失われる
- **ゾンビプロセス**: サーバー終了後も子プロセスが動き続ける
- **リソースリーク**: プロセスが適切に終了されず、リソースが解放されない

## ユースケース

### UC-001: 開発セッション終了（プロセス継続）

**Actor**: 開発者

**Goal**: Vantage を終了しても、開発サーバーは動き続けてほしい

**前提**: `VANTAGE_STOP_ON_SHUTDOWN=false`（デフォルト）

**Flow**:
1. Vantage MCP サーバーを Ctrl+C で終了
2. プロセス設定がスナップショットに保存される
3. Running 状態のプロセスはそのまま継続
4. 次回 Vantage 起動時にプロセス状態を再取得

**Success Criteria**: 開発サーバーが中断されず、作業を続行できる

### UC-002: クリーンシャットダウン（プロセス停止）

**Actor**: DevOps エンジニア

**Goal**: Vantage 終了時に全てのプロセスを停止し、クリーンな状態にしたい

**前提**: `VANTAGE_STOP_ON_SHUTDOWN=true`

**Flow**:
1. Vantage MCP サーバーを終了
2. Running 状態のプロセスに SIGTERM を送信
3. 猶予期間（3 秒）待機
4. まだ生きているプロセスに SIGKILL 送信
5. プロセス設定をスナップショットに保存
6. Vantage 終了

**Success Criteria**: 全てのプロセスが停止し、リソースが解放される

### UC-003: 緊急終了（強制終了）

**Actor**: 開発者

**Goal**: Vantage がフリーズした場合、強制終了したい

**Flow**:
1. Ctrl+C を複数回押下（または kill -9）
2. Vantage が即座に終了
3. プロセスは継続（停止処理がスキップされる）
4. 次回起動時にスナップショットを復元

**Success Criteria**: Vantage が即座に終了し、システムが応答する

## 機能要件

### FR-001: グレースフルシャットダウン（必須）

- **トリガー**: SIGTERM, SIGINT（Ctrl+C）
- **処理フロー**:
  1. シャットダウンシグナルをキャッチ
  2. 新しいリクエストの受付を停止
  3. 進行中のリクエストの完了を待機（最大 5 秒）
  4. プロセス停止処理（`VANTAGE_STOP_ON_SHUTDOWN` に依存）
  5. スナップショット保存
  6. サーバー終了

### FR-002: プロセス停止モードの選択（必須）

**環境変数**: `VANTAGE_STOP_ON_SHUTDOWN`

- **`false`（デフォルト）**: プロセスを停止せず、継続させる
  - プロセス状態はそのまま保存
  - 次回起動時に PID が変わっている可能性を考慮

- **`true`**: 全ての Running プロセスを停止
  - 各プロセスに SIGTERM を送信
  - 猶予期間（`VANTAGE_SHUTDOWN_GRACE_PERIOD_MS`）待機
  - まだ生きているプロセスに SIGKILL 送信

### FR-003: スナップショット自動保存（必須）

- **説明**: 終了時にプロセス設定を自動的に保存
- **保存先**:
  - SurrealDB（利用可能な場合）
  - YAML ファイル（`~/.vantage/snapshot.yaml`）
- **保存内容**:
  - プロセス設定（ID, command, args, env, cwd, auto_start_on_restore）
  - プロセス状態（NotStarted/Stopped/Failed のみ。Running は除外）
  - 作成日時、起動日時

### FR-004: エクスポートファイルパスの指定（オプション）

**環境変数**: `VANTAGE_EXPORT_FILE`

- **説明**: 終了時のスナップショット保存先を指定
- **デフォルト**: `~/.vantage/data/processes.surql`（SurrealDB が利用可能な場合）
- **例**:
  ```bash
  VANTAGE_EXPORT_FILE=/tmp/vantage-backup.yaml vantagemcp
  ```

### FR-005: 強制終了への対応（必須）

- **トリガー**: SIGKILL（kill -9）
- **処理**:
  - シグナルをキャッチできないため、即座に終了
  - スナップショット保存は実行されない
  - 次回起動時に最後の自動エクスポートから復元

## 非機能要件

### NFR-001: シャットダウン時間

- **目標**: 通常終了は 5 秒以内
- **測定**: シャットダウン開始から終了までの時間をログに記録

### NFR-002: データ完全性

- **目標**: スナップショット保存に失敗した場合でも、前回のスナップショットが残る
- **保証**: アトミックファイル書き込み（一時ファイル → rename）

### NFR-003: プロセス停止の信頼性

- **目標**: 95% のプロセスが SIGTERM で正常終了
- **フォールバック**: SIGKILL で強制終了

## 設計

### シャットダウンシーケンス

```rust
// crates/vantage-atom/src/lib.rs
impl VantageServer {
    pub async fn shutdown(&mut self) -> Result<(), String> {
        tracing::info!("Shutting down Vantage MCP Server...");

        // 1. 新しいリクエストの受付を停止
        // （実装はサーバーレイヤーで行う）

        // 2. プロセス停止処理
        if env::var("VANTAGE_STOP_ON_SHUTDOWN").unwrap_or_else(|_| "false".to_string()) == "true" {
            self.stop_all_processes().await?;
        }

        // 3. スナップショット保存
        self.export_snapshot().await?;

        tracing::info!("Vantage MCP Server shut down successfully");
        Ok(())
    }

    async fn stop_all_processes(&mut self) -> Result<(), String> {
        let running_processes = self.process_manager
            .list_processes()
            .await
            .into_iter()
            .filter(|p| matches!(p.state, ProcessState::Running { .. }))
            .collect::<Vec<_>>();

        for process in running_processes {
            match self.process_manager.stop_process(&process.id, Some(3000)).await {
                Ok(_) => {
                    tracing::info!("Stopped process: {}", process.id);
                }
                Err(e) => {
                    tracing::error!("Failed to stop process {}: {}", process.id, e);
                }
            }
        }

        Ok(())
    }

    async fn export_snapshot(&self) -> Result<(), String> {
        let export_file = env::var("VANTAGE_EXPORT_FILE")
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.vantage/data/processes.surql", home)
            });

        // アトミック書き込み
        let temp_file = format!("{}.tmp", export_file);
        self.persistence.export_to_file(&temp_file).await?;
        fs::rename(&temp_file, &export_file)
            .map_err(|e| format!("Failed to rename snapshot: {}", e))?;

        tracing::info!("Snapshot saved to: {}", export_file);
        Ok(())
    }
}
```

### シグナルハンドリング

```rust
// crates/vantage-mcp/src/main.rs
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = VantageServer::new().await?;

    // Ctrl+C ハンドラー
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        tracing::info!("Received Ctrl+C signal");
        server.shutdown().await.expect("Failed to shutdown");
        std::process::exit(0);
    });

    server.run().await?;
    Ok(())
}
```

### 終了ログ例

**VANTAGE_STOP_ON_SHUTDOWN=false（デフォルト）**:
```
2025-11-02T18:00:00Z INFO  Received Ctrl+C signal
2025-11-02T18:00:00Z INFO  Shutting down Vantage MCP Server...
2025-11-02T18:00:00Z INFO  Skipping process shutdown (VANTAGE_STOP_ON_SHUTDOWN=false)
2025-11-02T18:00:00Z INFO  Exporting snapshot to ~/.vantage/data/processes.surql
2025-11-02T18:00:01Z INFO  Snapshot saved (5 processes)
2025-11-02T18:00:01Z INFO  Vantage MCP Server shut down successfully
```

**VANTAGE_STOP_ON_SHUTDOWN=true**:
```
2025-11-02T18:00:00Z INFO  Received Ctrl+C signal
2025-11-02T18:00:00Z INFO  Shutting down Vantage MCP Server...
2025-11-02T18:00:00Z INFO  Stopping all running processes...
2025-11-02T18:00:00Z INFO  Stopped process: frontend-dev (PID: 12345)
2025-11-02T18:00:01Z INFO  Stopped process: backend-api (PID: 12346)
2025-11-02T18:00:02Z INFO  Stopped process: postgres-db (PID: 12347)
2025-11-02T18:00:02Z INFO  Exporting snapshot to ~/.vantage/data/processes.surql
2025-11-02T18:00:03Z INFO  Snapshot saved (5 processes)
2025-11-02T18:00:03Z INFO  Vantage MCP Server shut down successfully
```

## 環境変数

| 変数 | 説明 | デフォルト |
|------|------|------------|
| `VANTAGE_STOP_ON_SHUTDOWN` | 終了時にプロセスを停止するか | false（継続） |
| `VANTAGE_SHUTDOWN_GRACE_PERIOD_MS` | SIGTERM 後の猶予期間（ms） | 3000 |
| `VANTAGE_EXPORT_FILE` | スナップショット保存先 | `~/.vantage/data/processes.surql` |

**使用例**:
```bash
# 終了時にプロセスを停止
VANTAGE_STOP_ON_SHUTDOWN=true vantagemcp

# 猶予期間を 5 秒に延長
VANTAGE_SHUTDOWN_GRACE_PERIOD_MS=5000 vantagemcp

# カスタム保存先
VANTAGE_EXPORT_FILE=/backup/vantage.yaml vantagemcp
```

## テストケース

### TC-001: 通常終了（プロセス継続）

**前提**: `VANTAGE_STOP_ON_SHUTDOWN=false`

**操作**: Vantage を Ctrl+C で終了

**期待結果**:
- プロセスは停止されず、継続
- スナップショットが保存される
- Vantage が終了

### TC-002: 通常終了（プロセス停止）

**前提**: `VANTAGE_STOP_ON_SHUTDOWN=true`

**操作**: Vantage を Ctrl+C で終了

**期待結果**:
- 全ての Running プロセスに SIGTERM 送信
- 3 秒待機後、まだ生きているプロセスに SIGKILL 送信
- スナップショットが保存される
- Vantage が終了

### TC-003: 強制終了（kill -9）

**操作**: `kill -9 <vantage_pid>`

**期待結果**:
- Vantage が即座に終了
- スナップショット保存はスキップされる
- プロセスは継続
- 次回起動時に前回のスナップショットを復元

### TC-004: スナップショット保存の失敗

**前提**: 保存先ディレクトリに書き込み権限がない

**操作**: Vantage を終了

**期待結果**:
- エラーログが記録される
- 前回のスナップショットが残る
- Vantage は終了（エラーでも終了を妨げない）

### TC-005: 自動エクスポート間隔

**前提**: `VANTAGE_AUTO_EXPORT_INTERVAL=60`（60 秒ごと）

**操作**:
1. Vantage を起動
2. 60 秒待機
3. 強制終了（kill -9）

**期待結果**:
- 60 秒後に自動エクスポートが実行される
- 強制終了してもスナップショットが保存されている

## 制約

- **SIGKILL 耐性なし**: kill -9 ではスナップショット保存が実行されない
- **ネットワーク障害**: SurrealDB への保存失敗時は YAML フォールバック

## 今後の拡張

### プロセスグループの停止

特定のグループのみを停止：
```yaml
shutdown:
  stop_groups:
    - development  # 開発環境のみ停止
  continue_groups:
    - infrastructure  # DB などは継続
```

### 終了前フック

カスタムスクリプトを実行：
```yaml
hooks:
  before_shutdown:
    - command: "npm run build"
      working_dir: "/app"
```

### 自動バックアップ

終了時に複数世代のバックアップを保存：
```yaml
backup:
  enabled: true
  retention: 5  # 最新 5 世代を保持
  directory: "~/.vantage/backups"
```

## 更新履歴

| 日付 | 変更者 | 変更内容 | 理由 |
|------|--------|----------|------|
| 2025-11-02 | Claude Code | 初版作成 | 仕様の明確化 |
