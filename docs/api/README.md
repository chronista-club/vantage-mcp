# Ichimi Server API リファレンス

## MCP ツール API

### プロセス管理

#### `create_process`
プロセス設定を作成します。

**パラメータ:**
- `process_id` (string): プロセスの一意識別子
- `name` (string): プロセス名
- `command` (string): 実行するコマンド
- `args` (array): コマンド引数
- `env` (object, optional): 環境変数
- `cwd` (string, optional): 作業ディレクトリ
- `auto_start_on_restore` (boolean, optional): 復元時の自動起動

**レスポンス:**
```json
{
  "process_id": "my-process",
  "status": "created"
}
```

#### `start_process`
作成済みプロセスを起動します。

**パラメータ:**
- `process_id` (string): プロセスID

**レスポンス:**
```json
{
  "process_id": "my-process",
  "pid": 12345,
  "status": "running"
}
```

#### `stop_process`
実行中のプロセスを停止します。

**パラメータ:**
- `process_id` (string): プロセスID
- `timeout_ms` (number, optional): タイムアウト（ミリ秒）

**レスポンス:**
```json
{
  "process_id": "my-process",
  "exit_code": 0,
  "status": "stopped"
}
```

#### `get_process_status`
プロセスの現在の状態を取得します。

**パラメータ:**
- `process_id` (string): プロセスID

**レスポンス:**
```json
{
  "process_id": "my-process",
  "state": "running",
  "pid": 12345,
  "started_at": "2025-09-28T10:00:00Z",
  "cpu_usage": 15.2,
  "memory_usage": 102400
}
```

#### `get_process_output`
プロセスの出力を取得します。

**パラメータ:**
- `process_id` (string): プロセスID
- `limit` (number, optional): 取得する行数の上限

**レスポンス:**
```json
{
  "process_id": "my-process",
  "output": [
    "[2025-09-28 10:00:00] Starting process...",
    "[2025-09-28 10:00:01] Process initialized"
  ]
}
```

#### `list_processes`
すべてのプロセスをリストします。

**パラメータ:**
- `filter` (string, optional): フィルター ("running", "stopped", "failed", "all")

**レスポンス:**
```json
{
  "processes": [
    {
      "process_id": "process-1",
      "name": "Web Server",
      "state": "running",
      "pid": 12345
    },
    {
      "process_id": "process-2",
      "name": "Worker",
      "state": "stopped",
      "exit_code": 0
    }
  ]
}
```

### CI/CD 監視

#### `list_ci_runs`
最近のCI実行をリストします。

**パラメータ:**
- `limit` (number, optional): 取得する実行数の上限

**レスポンス:**
```json
{
  "runs": [
    {
      "id": 123456,
      "workflow_name": "CI",
      "branch": "main",
      "status": "completed",
      "conclusion": "success",
      "url": "https://github.com/..."
    }
  ]
}
```

#### `wait_for_ci_completion`
CI実行の完了を待機します。

**パラメータ:**
- `run_id` (number): CI実行ID
- `timeout_secs` (number, optional): タイムアウト（秒）

**レスポンス:**
```json
{
  "run_id": 123456,
  "status": "completed",
  "conclusion": "success",
  "duration": "2m 30s"
}
```

## REST API

Webダッシュボードが使用する内部APIです。

### エンドポイント

#### `GET /api/processes`
すべてのプロセスを取得

#### `POST /api/processes`
新しいプロセスを作成

#### `GET /api/processes/{id}`
特定のプロセスの詳細を取得

#### `POST /api/processes/{id}/start`
プロセスを開始

#### `POST /api/processes/{id}/stop`
プロセスを停止

#### `DELETE /api/processes/{id}`
プロセスを削除

#### `GET /api/processes/{id}/output`
プロセスの出力を取得

#### `GET /api/system/status`
システムステータスを取得

## エラーコード

| コード | 説明 |
|-------|------|
| `PROCESS_NOT_FOUND` | 指定されたプロセスが見つかりません |
| `PROCESS_ALREADY_EXISTS` | プロセスIDが既に使用されています |
| `PROCESS_NOT_RUNNING` | プロセスが実行されていません |
| `INVALID_COMMAND` | 無効なコマンドまたは引数 |
| `PERMISSION_DENIED` | 権限が不足しています |
| `INTERNAL_ERROR` | サーバー内部エラー |