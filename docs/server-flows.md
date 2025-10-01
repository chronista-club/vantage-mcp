# MCP サーバー起動フロー

このドキュメントは、Ichimi MCP サーバーの起動から起動完了までのフローを説明します。

## 起動フロー概要

```mermaid
flowchart TD
    Start([プログラム開始]) --> ParseArgs[コマンドライン引数解析]
    ParseArgs --> CheckMode{モード判定}

    CheckMode -->|--help| ShowHelp[ヘルプ表示]
    CheckMode -->|--version| ShowVersion[バージョン表示]
    CheckMode -->|その他| InitLogging[ロギング設定]

    ShowHelp --> End([終了])
    ShowVersion --> End

    InitLogging --> LogMode{ログ出力先}
    LogMode -->|MCPモード| FileLog[ファイルログ<br/>~/.ichimi/logs/]
    LogMode -->|Webモード| StderrLog[標準エラー出力]

    FileLog --> InitPM[ProcessManager初期化]
    StderrLog --> InitPM

    InitPM --> InitPersistence[PersistenceManager初期化]
    InitPersistence --> CreateHashMap[プロセスHashMap作成]
    CreateHashMap --> RestoreSnapshot{スナップショット存在?}

    RestoreSnapshot -->|Yes| LoadYAML[YAMLスナップショット読込]
    RestoreSnapshot -->|No| SetupSignal[シグナルハンドラー設定]
    LoadYAML --> RestoreProcesses[プロセス情報リストア]
    RestoreProcesses --> SetupSignal

    SetupSignal --> CheckWeb{Webモード?}
    CheckWeb -->|Yes| StartWeb[Webサーバー起動]
    CheckWeb -->|No| CheckMCP
    StartWeb --> OpenBrowser{ブラウザ自動起動?}
    OpenBrowser -->|Yes| LaunchBrowser[ブラウザ起動]
    OpenBrowser -->|No| CheckMCP
    LaunchBrowser --> CheckMCP

    CheckMCP{MCPサーバー起動?}
    CheckMCP -->|Yes| InitServer[IchimiServer初期化]
    CheckMCP -->|No| KeepAlive[プロセス維持ループ]

    InitServer --> InitEvent[EventSystem初期化]
    InitEvent --> InitLearning[LearningEngine初期化]
    InitLearning --> StartLearning[学習エンジン起動]
    StartLearning --> InitCI[CiMonitor初期化]
    InitCI --> CreateRouter[ToolRouter作成]
    CreateRouter --> ServeStdio[STDIOトランスポートで<br/>MCPサーバー起動]

    ServeStdio --> WaitRequests[クライアントリクエスト待機]
    WaitRequests --> Ready([起動完了])

    KeepAlive --> Ready
```

## 詳細フロー

### 1. コマンドライン引数解析

サポートされるオプション：
- `--help`, `-h`: ヘルプ表示
- `--version`, `-v`: バージョン表示
- `--web`: Web ダッシュボードを MCP サーバーと並行起動
- `--web-only`: MCP サーバーなしで Web ダッシュボードのみ起動
- `--web-port PORT`: Web ダッシュボードのポート指定（デフォルト: 12700）
- `--no-open`: ブラウザ自動起動を無効化
- `--app-mode`: ブラウザをアプリモード（専用ウィンドウ）で起動

### 2. ロギング設定

起動モードによってログ出力先が変わります：

#### MCP モード（デフォルト）
- **出力先**: `~/.ichimi/logs/ichimi-mcp-YYYYMMDD-HHMMSS.log`
- **理由**: stdio を MCP 通信に使用するため、ログは別ファイルに出力
- **設定**: ファイルアペンダー、ANSI カラー無効

#### Web モードまたは MCP+Web モード
- **出力先**: 標準エラー出力（stderr）
- **設定**: ANSI カラー無効

### 3. ProcessManager 初期化

```mermaid
sequenceDiagram
    participant Main
    participant PM as ProcessManager
    participant Persist as PersistenceManager
    participant KDL as KDL Storage

    Main->>PM: new()
    PM->>Persist: new()
    Persist->>KDL: 初期化
    KDL->>KDL: ~/.ichimi/processes.kdl 確認
    KDL-->>Persist: OK
    Persist-->>PM: Arc<PersistenceManager>
    PM->>PM: HashMap<String, Process> 作成
    PM-->>Main: ProcessManager
```

**初期化内容**：
- `PersistenceManager` の初期化
  - KDL 形式のストレージ初期化
  - データディレクトリ: `~/.ichimi/`
  - 設定ファイル: `~/.ichimi/processes.kdl`
- プロセス管理用 `HashMap` の作成（`Arc<RwLock<HashMap>>`）

### 4. プロセスリストア

YAMLスナップショットからプロセス情報をリストアします：

```mermaid
sequenceDiagram
    participant Main
    participant PM as ProcessManager
    participant Persist as PersistenceManager
    participant YAML as YAML File

    Main->>YAML: ~/.ichimi/snapshot.yaml 存在確認
    alt ファイル存在
        Main->>PM: restore_yaml_snapshot()
        PM->>Persist: load YAML
        Persist->>YAML: 読み込み
        YAML-->>Persist: プロセス情報
        Persist->>Persist: デシリアライズ
        Persist-->>PM: Vec<ProcessInfo>
        PM->>PM: auto_start_on_restore フラグ確認
        PM->>PM: プロセス情報を HashMap に登録
        PM-->>Main: OK
    else ファイルなし
        Main->>Main: スキップ
    end
```

**リストア内容**：
- プロセス定義（ID、コマンド、引数、環境変数など）
- `auto_start_on_restore` フラグが `true` のプロセスは起動対象としてマーク

### 5. シグナルハンドラー設定

グレースフルシャットダウンのためのシグナルハンドラーを設定：

- **Unix系**: SIGINT (Ctrl+C), SIGTERM
- **Windows**: Ctrl+C

シグナル受信時の処理：
1. YAML スナップショット作成
2. すべてのプロセスを停止
3. ブラウザウィンドウを閉じる（app-mode の場合）
4. プログラム終了

### 6. Web サーバー起動（オプション）

`--web` または `--web-only` フラグが指定された場合：

```mermaid
sequenceDiagram
    participant Main
    participant Web as Web Server
    participant Browser

    Main->>Web: start_web_server(port)
    Web->>Web: ポート 12700 で bind 試行
    alt バインド成功
        Web-->>Main: actual_port
    else ポート使用中
        Web->>Web: 次の空きポートを検索
        Web-->>Main: actual_port
    end

    opt auto_open = true
        Main->>Main: 500ms 待機
        alt app-mode
            Main->>Browser: Chrome/Safari をアプリモードで起動
            Browser-->>Main: PID
        else 通常モード
            Main->>Browser: デフォルトブラウザで起動
        end
    end
```

**Web サーバーの機能**：
- プロセス管理 Web UI
- RESTful API
- リアルタイムログ表示

### 7. IchimiServer 初期化

MCP サーバーのコア機能を初期化：

```mermaid
sequenceDiagram
    participant Main
    participant Server as IchimiServer
    participant Event as EventSystem
    participant Learning as LearningEngine
    participant CI as CiMonitor

    Main->>Server: with_process_manager(pm)
    Server->>Event: new()
    Event-->>Server: EventSystem

    Server->>Learning: new(event_system)
    Learning-->>Server: LearningEngine

    Server->>Learning: start_learning()
    Learning->>Learning: 学習タスク起動
    Learning-->>Server: OK

    Server->>CI: new(None, Some(30))
    CI-->>Server: CiMonitor

    Server->>Server: tool_router() マクロ実行
    Server->>Server: MCP ツール登録

    Server-->>Main: IchimiServer
```

**初期化コンポーネント**：
- **EventSystem**: イベント駆動アーキテクチャのためのイベントバス
- **LearningEngine**: プロセス動作の学習と最適化
- **CiMonitor**: CI/CD パイプライン監視（ポーリング間隔: 30秒）
- **ToolRouter**: MCP ツールのルーティング（`#[tool_router]` マクロで自動生成）

### 8. MCP サーバー起動

STDIO トランスポートで MCP サーバーを起動：

```mermaid
sequenceDiagram
    participant Main
    participant Server as IchimiServer
    participant MCP as MCP Service
    participant Client as Claude Code

    Main->>Server: serve(stdio())
    Server->>MCP: 起動
    MCP->>MCP: STDIO で通信準備
    MCP-->>Server: Service

    Server->>MCP: waiting()
    Note over MCP: リクエスト待機状態

    Client->>MCP: initialize リクエスト
    MCP->>Server: get_info()
    Server-->>MCP: ServerInfo
    MCP-->>Client: 初期化完了

    Note over MCP,Client: MCP サーバー稼働中
```

**通信方式**：
- **トランスポート**: STDIO（標準入出力）
- **プロトコル**: JSON-RPC 2.0 over MCP
- **プロトコルバージョン**: 2024-11-05

### 9. 起動完了

サーバーは以下の状態で待機：
- MCP クライアント（Claude Code）からのリクエストを受付
- Web ダッシュボード（有効な場合）でリアルタイム管理可能
- シグナル受信時はグレースフルシャットダウン

## 起動モード

### MCP モード（デフォルト）
```bash
ichimi
```
- MCP サーバーのみ起動
- Claude Code から利用可能
- ログはファイルに出力

### MCP + Web モード
```bash
ichimi --web
```
- MCP サーバーと Web ダッシュボードの両方起動
- Claude Code と Web UI の両方から管理可能

### Web のみモード
```bash
ichimi --web-only
```
- Web ダッシュボードのみ起動
- Claude Code からは利用不可
- スタンドアロンプロセスマネージャーとして動作

## 環境変数

| 変数 | 説明 | デフォルト |
|------|------|------------|
| `RUST_LOG` | ログレベル (error, warn, info, debug, trace) | info |
| `ICHIMI_IMPORT_FILE` | 起動時にインポートするファイル | ~/.ichimi/snapshot.yaml |
| `ICHIMI_EXPORT_FILE` | シャットダウン時のエクスポート先 | ~/.ichimi/snapshot.yaml |
| `ICHIMI_DATA_DIR` | データファイル用ディレクトリ | ~/.ichimi/ |
| `ICHIMI_STOP_ON_SHUTDOWN` | ichimi終了時にプロセスを停止するか（true/false） | false（継続） |

## ディレクトリ構造

```
~/.ichimi/
├── processes.kdl          # KDL形式のプロセス設定
├── snapshot.yaml          # YAMLスナップショット（バックアップ/リストア用）
├── logs/                  # ログディレクトリ（MCPモード時）
│   └── ichimi-mcp-YYYYMMDD-HHMMSS.log
└── data/                  # その他のデータファイル
```

---

## シャットダウンフロー

Ichimi サーバーの終了処理は、管理プロセスのクリーンアップとデータ保存を確実に行います。

```mermaid
flowchart TD
    Signal([シグナル受信<br/>SIGINT/SIGTERM]) --> CreateSnapshot[YAMLスナップショット作成]
    CreateSnapshot --> SaveAutoStart[auto_start_on_restore<br/>フラグ付きプロセスを保存]
    SaveAutoStart --> StopAll[全プロセスを停止]

    StopAll --> CheckProcess{プロセス実行中?}
    CheckProcess -->|Yes| StopProcess[各プロセスを停止]
    CheckProcess -->|No| CloseWebDriver

    StopProcess --> SendSIGTERM[プロセスグループに<br/>SIGTERM送信]
    SendSIGTERM --> WaitGrace[グレースピリオド待機<br/>デフォルト: 5秒]
    WaitGrace --> CheckExit{プロセス終了?}

    CheckExit -->|Yes| NextProcess[次のプロセスへ]
    CheckExit -->|No| SendSIGKILL[プロセスグループに<br/>SIGKILL送信]
    SendSIGKILL --> ForceWait[強制終了待機<br/>タイムアウト: 10秒]
    ForceWait --> NextProcess

    NextProcess --> CheckMore{他のプロセス?}
    CheckMore -->|Yes| StopProcess
    CheckMore -->|No| CloseWebDriver

    CloseWebDriver{WebDriverモード?}
    CloseWebDriver -->|Yes| CloseWD[WebDriverセッション終了]
    CloseWebDriver -->|No| CloseBrowser
    CloseWD --> CloseBrowser

    CloseBrowser{ブラウザ起動中?}
    CloseBrowser -->|Yes, App Mode| SendBrowserSIGTERM[ブラウザに<br/>SIGTERM送信]
    CloseBrowser -->|No| Exit

    SendBrowserSIGTERM --> WaitBrowser[1秒待機]
    WaitBrowser --> CheckBrowserExit{ブラウザ終了?}
    CheckBrowserExit -->|Yes| Exit
    CheckBrowserExit -->|No| KillBrowser[ブラウザ強制終了]
    KillBrowser --> Exit

    Exit([プロセス終了<br/>exit 0])
```

### シャットダウン詳細シーケンス

```mermaid
sequenceDiagram
    participant Signal as シグナル
    participant Handler as SignalHandler
    participant PM as ProcessManager
    participant Process as 管理対象プロセス
    participant Browser

    Signal->>Handler: SIGINT/SIGTERM

    Note over Handler: 1. スナップショット作成
    Handler->>PM: create_auto_start_snapshot()
    PM->>PM: auto_start_on_restore=true<br/>のプロセスをフィルタ
    PM->>PM: ~/.ichimi/snapshot.yaml に保存
    PM-->>Handler: OK

    Note over Handler: 2. プロセス停止
    Handler->>PM: stop_all_processes()

    loop 各プロセス
        PM->>Process: プロセスグループにSIGTERM送信
        PM->>PM: 5秒待機

        alt プロセス終了
            Process-->>PM: 終了
        else タイムアウト
            PM->>Process: プロセスグループにSIGKILL送信
            PM->>PM: 10秒待機（タイムアウト付き）

            alt 終了成功
                Process-->>PM: 強制終了
            else 終了失敗
                PM->>PM: エラーログ記録
            end
        end
    end

    PM-->>Handler: 停止したプロセスリスト

    Note over Handler: 3. ブラウザクローズ
    opt app-mode で起動した場合
        Handler->>Browser: SIGTERM送信
        Handler->>Handler: 1秒待機

        alt ブラウザ終了
            Browser-->>Handler: 終了
        else まだ実行中
            Handler->>Browser: SIGKILL
            Browser-->>Handler: 強制終了
        end
    end

    Handler->>Handler: exit(0)
```

### シャットダウンの重要ポイント

1. **スナップショット作成**
   - `auto_start_on_restore` フラグが true のプロセスのみ保存
   - 次回起動時に自動復元される

2. **プロセス停止**
   - すべての管理プロセスを停止（環境変数 `ICHIMI_STOP_ON_SHUTDOWN` に関係なく常に実行）
   - プロセスグループ単位でシグナル送信（Docker対応）
   - グレースフルシャットダウンを試みた後、必要に応じて強制終了

3. **ブラウザクリーンアップ**
   - app-mode で起動したブラウザウィンドウは確実に終了

---

## 重要な機能フロー

### プロセス起動フロー（プロセスグループ対応）

v0.2.1 で改善されたプロセス起動フローです。Docker などの子プロセスを持つプロセスを確実に停止できます。

```mermaid
sequenceDiagram
    participant Client as Claude Code
    participant MCP as MCP Server
    participant PM as ProcessManager
    participant OS as OS Process
    participant Child as 子プロセス<br/>(Docker等)

    Client->>MCP: start_process("my-docker")
    MCP->>PM: start_process(id)

    PM->>PM: プロセス設定取得
    PM->>PM: Command::new(command)
    PM->>PM: .args(args)<br/>.env(env_vars)<br/>.cwd(working_dir)

    Note over PM: プロセスグループ設定
    PM->>PM: process_group(0)<br/>※新しいグループのリーダーに

    PM->>OS: spawn()
    OS->>OS: 新しいプロセスグループ作成<br/>PGID = PID
    OS-->>PM: Child + PID

    PM->>PM: stdout/stderr パイプ設定
    PM->>PM: 出力キャプチャタスク起動
    PM->>PM: 状態を Running に更新

    OS->>Child: 子プロセス起動<br/>(同じプロセスグループに所属)

    PM-->>MCP: PID
    MCP-->>Client: success

    Note over OS,Child: プロセスとその子プロセスが<br/>同じプロセスグループに所属
```

**重要な改善点（v0.2.1）**：
- `process_group(0)` で新しいプロセスグループのリーダーとして起動
- 子プロセス（Docker コンテナなど）も同じプロセスグループに所属
- 停止時にプロセスグループ全体にシグナル送信可能

### プロセス停止フロー（Docker 対応）

Docker プロセスとコンテナを確実に停止する改善されたフローです。

```mermaid
sequenceDiagram
    participant Client as Claude Code
    participant MCP as MCP Server
    participant PM as ProcessManager
    participant OS as OS Process
    participant Docker as Docker Container

    Client->>MCP: stop_process("my-docker")
    MCP->>PM: stop_process(id, grace_period)

    PM->>PM: プロセス情報取得
    PM->>PM: PIDを取得
    PM->>PM: PGID = -PID として計算

    Note over PM,OS: グレースフルシャットダウン試行
    PM->>OS: kill(-PGID, SIGTERM)<br/>※プロセスグループ全体に送信
    OS->>OS: メインプロセスにSIGTERM
    OS->>Docker: 子プロセスにもSIGTERM

    PM->>PM: グレースピリオド待機<br/>デフォルト: 5秒

    alt プロセスが終了
        OS-->>PM: 終了ステータス
        Docker-->>OS: コンテナ停止
        PM->>PM: 状態を Stopped に更新
        PM-->>MCP: success
    else タイムアウト
        Note over PM,OS: 強制終了
        PM->>OS: kill(-PGID, SIGKILL)<br/>※プロセスグループ全体に送信
        OS->>OS: メインプロセスにSIGKILL
        OS->>Docker: 子プロセスにもSIGKILL

        PM->>PM: 10秒待機（タイムアウト付き）

        alt 終了成功
            OS-->>PM: 強制終了完了
            Docker-->>OS: コンテナ強制停止
            PM->>PM: 状態を Stopped に更新
            PM-->>MCP: success
        else 終了失敗
            PM-->>MCP: error("タイムアウト")
        end
    end

    MCP-->>Client: result
```

**プロセスグループシグナルの効果**：
1. 負の PID（`-PGID`）でシグナル送信
2. プロセスグループ全体（メイン + 子プロセス）にシグナルが届く
3. Docker の場合：
   - `docker run` プロセスが SIGTERM を受信
   - Docker がコンテナにシグナルを転送
   - コンテナとプロセスの両方が終了

### Web サーバー連携フロー

```mermaid
sequenceDiagram
    participant UI as Web UI
    participant API as Web Server API
    participant PM as ProcessManager
    participant Persist as PersistenceManager

    Note over UI,Persist: プロセス一覧取得
    UI->>API: GET /api/processes
    API->>PM: list_processes()
    PM->>PM: プロセスHashMap読み取り
    PM-->>API: Vec<ProcessInfo>
    API-->>UI: JSON response

    Note over UI,Persist: プロセス起動
    UI->>API: POST /api/processes/:id/start
    API->>PM: start_process(id)
    PM->>PM: プロセス起動処理
    PM->>Persist: update_process(info)
    Persist->>Persist: KDL ファイルに保存
    PM-->>API: PID
    API-->>UI: success

    Note over UI,Persist: リアルタイムログ取得
    UI->>API: GET /api/processes/:id/output
    API->>PM: get_process_output(id, lines)
    PM->>PM: CircularBuffer から取得
    PM-->>API: stdout/stderr
    API-->>UI: JSON response
```

### スナップショット保存・復元フロー

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant PM as ProcessManager
    participant Persist as PersistenceManager
    participant YAML as snapshot.yaml

    Note over User,YAML: シャットダウン時の保存
    User->>PM: シャットダウンシグナル
    PM->>PM: auto_start_on_restore=true<br/>のプロセスをフィルタ
    PM->>Persist: create_snapshot(processes)
    Persist->>YAML: シリアライズして保存
    YAML-->>Persist: OK
    Persist-->>PM: OK

    Note over User,YAML: 起動時の復元
    User->>PM: ichimi 起動
    PM->>YAML: ~/.ichimi/snapshot.yaml 存在確認
    alt ファイル存在
        PM->>Persist: restore_yaml_snapshot()
        Persist->>YAML: 読み込み
        YAML-->>Persist: プロセス情報
        Persist->>Persist: デシリアライズ
        Persist-->>PM: Vec<ProcessInfo>

        loop 各プロセス
            PM->>PM: プロセス登録
            alt auto_start_on_restore = true
                PM->>PM: 起動キューに追加
                PM->>PM: start_process(id)
            end
        end

        PM-->>User: 起動完了
    else ファイルなし
        PM-->>User: 新規起動
    end
```

## プロセス状態遷移図

```mermaid
stateDiagram-v2
    [*] --> NotStarted: create_process
    NotStarted --> Running: start_process
    Running --> Stopped: stop_process (正常終了)
    Running --> Failed: プロセスクラッシュ
    Stopped --> Running: start_process (再起動)
    Failed --> Running: start_process (再起動)
    Stopped --> [*]: remove_process
    Failed --> [*]: remove_process
    NotStarted --> [*]: remove_process

    note right of NotStarted
        プロセス定義は存在するが
        まだ起動されていない状態
    end note

    note right of Running
        PID が割り当てられ
        プロセスが実行中
        stdout/stderr をキャプチャ
    end note

    note right of Stopped
        正常に停止された状態
        exit_code を保持
    end note

    note right of Failed
        異常終了した状態
        エラー詳細を保持
    end note
```

## 関連ドキュメント

- [CLAUDE.md](../.claude/CLAUDE.md) - プロジェクト概要とアーキテクチャ
- [README.md](../README.md) - ユーザー向けドキュメント
