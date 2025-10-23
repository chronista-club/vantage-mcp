use anyhow::Result;
use clap::Parser;
use ichimi::IchimiServer;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{self, EnvFilter};

// メンテナビリティ向上のための定数
const BROWSER_STARTUP_DELAY_MS: u64 = 500;
const BROWSER_SHUTDOWN_GRACE_MS: u64 = 1000;
const KEEPALIVE_INTERVAL_SECS: u64 = 3600;

/// Ichimi Server - MCP経由のClaude Code用プロセス管理サーバー
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// MCPサーバーと並行してWebダッシュボードを有効化
    #[arg(long)]
    web: bool,

    /// Webダッシュボードのみを実行（MCPサーバーなし）
    #[arg(long)]
    web_only: bool,

    /// Webダッシュボードのポートを設定
    #[arg(long, default_value_t = 12700)]
    web_port: u16,

    /// Webダッシュボード用のブラウザを自動的に開かない
    #[arg(long)]
    no_open: bool,

    /// ブラウザをアプリモードで開く（サーバーと連動して閉じる専用ウィンドウ）
    #[arg(long)]
    app_mode: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum DefaultBrowser {
    Chrome,
    Firefox,
    Safari,
    Unknown,
}

// システムのデフォルトブラウザを検出
#[allow(dead_code)]
fn detect_default_browser() -> DefaultBrowser {
    #[cfg(target_os = "macos")]
    {
        // macOSのデフォルトブラウザ設定を読み取ろうと試行
        let output = std::process::Command::new("plutil")
            .args(["-p", &format!("{}/Library/Preferences/com.apple.LaunchServices/com.apple.launchservices.secure.plist", env::var("HOME").unwrap_or_default())])
            .output();

        if output.is_err() {
            // 代替の場所を試行
            let output = std::process::Command::new("defaults")
                .args([
                    "read",
                    "com.apple.LaunchServices/com.apple.launchservices.secure",
                ])
                .output();

            if let Ok(output) = output {
                let content = String::from_utf8_lossy(&output.stdout);
                if content.contains("com.google.chrome") {
                    return DefaultBrowser::Chrome;
                } else if content.contains("org.mozilla.firefox") {
                    return DefaultBrowser::Firefox;
                } else if content.contains("com.apple.safari") {
                    return DefaultBrowser::Safari;
                }
            }
        } else if let Ok(output) = output {
            let content = String::from_utf8_lossy(&output.stdout);
            if content.contains("com.google.chrome") {
                return DefaultBrowser::Chrome;
            } else if content.contains("org.mozilla.firefox") {
                return DefaultBrowser::Firefox;
            } else if content.contains("com.apple.safari") {
                return DefaultBrowser::Safari;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // デフォルトブラウザのWindowsレジストリをチェック
        let output = std::process::Command::new("reg")
            .args(&["query", r"HKEY_CURRENT_USER\Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice", "/v", "ProgId"])
            .output();

        if let Ok(output) = output {
            let content = String::from_utf8_lossy(&output.stdout);
            if content.contains("ChromeHTML") {
                return DefaultBrowser::Chrome;
            } else if content.contains("FirefoxURL") {
                return DefaultBrowser::Firefox;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // デフォルトブラウザのxdg-settingsをチェック
        let output = std::process::Command::new("xdg-settings")
            .args(&["get", "default-web-browser"])
            .output();

        if let Ok(output) = output {
            let content = String::from_utf8_lossy(&output.stdout);
            if content.contains("chrome") || content.contains("chromium") {
                return DefaultBrowser::Chrome;
            } else if content.contains("firefox") {
                return DefaultBrowser::Firefox;
            }
        }
    }

    DefaultBrowser::Unknown
}

#[tokio::main]
async fn main() -> Result<()> {
    // clapを使用してコマンドライン引数をパース
    let cli = Cli::parse();

    // CLI引数から設定を導出
    let web_enabled = cli.web || cli.web_only;
    let web_only = cli.web_only;
    let web_port = cli.web_port;
    let auto_open = !cli.no_open;
    let app_mode = cli.app_mode;

    // ロギング用に引数を収集
    let args: Vec<String> = env::args().collect();

    // 動作モードを決定
    let run_mcp = !web_only; // --web-onlyが指定されていない限り、デフォルトでMCPサーバーを実行

    // 環境に基づいてロギングをセットアップ
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // MCP（デフォルト）として実行する場合、stdioとの干渉を避けるためファイルにログ出力
    if run_mcp && !web_enabled {
        // MCPとして実行する場合、stdioとの干渉を避けるためファイルにログ出力
        let log_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".ichimi")
            .join("logs");

        // ログディレクトリが存在しない場合は作成
        std::fs::create_dir_all(&log_dir).ok();

        // タイムスタンプ付きのログファイル名を生成
        let log_file = log_dir.join(format!(
            "ichimi-mcp-{}.log",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));

        // ファイルアペンダーを作成
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .map_err(|e| anyhow::anyhow!("Failed to create log file: {}", e))?;

        let filter = EnvFilter::from_default_env()
            .add_directive(
                format!("ichimi={log_level}")
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            )
            .add_directive(
                format!("ichimi_server={log_level}")
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            )
            .add_directive(
                "facet_kdl=warn"
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            )
            .add_directive(
                "mcp_server=debug"
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            );

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();

        tracing::info!("=== Ichimi MCP Server Starting (silent mode) ===");
        tracing::info!("Log file: {:?}", log_file);
        tracing::info!("Arguments: {:?}", args);
        tracing::info!("Working directory: {:?}", env::current_dir());
    } else {
        // WebモードまたはMCP+Webモード - stderrにログ出力
        let filter = EnvFilter::from_default_env()
            .add_directive(
                format!("ichimi={log_level}")
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            )
            .add_directive(
                format!("ichimi_server={log_level}")
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            )
            .add_directive(
                "facet_kdl=warn"
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
            );

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();

        if web_only {
            tracing::info!("🚀 Starting Ichimi Development Server (web-only mode)");
        } else if run_mcp && web_enabled {
            tracing::info!("Starting Ichimi Server (MCP + web mode)");
        } else {
            tracing::info!("Starting Ichimi Server (MCP mode)");
        }
    }

    // 共有プロセスマネージャーを作成
    let process_manager = ichimi::process::ProcessManager::new().await;

    // クリーンアップ用にブラウザプロセスを追跡
    let browser_process: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let browser_process_for_shutdown = browser_process.clone();

    // 設定されている場合、起動時にプロセスを自動インポート
    // まず自動起動プロセス用のYAMLスナップショットを試行
    let yaml_snapshot = std::env::var("HOME")
        .map(|home| format!("{home}/.ichimi/snapshot.yaml"))
        .unwrap_or_else(|_| ".ichimi/snapshot.yaml".to_string());

    if std::path::Path::new(&yaml_snapshot).exists() {
        tracing::info!("Restoring from YAML snapshot: {}", yaml_snapshot);
        match process_manager.restore_yaml_snapshot().await {
            Ok(_) => {
                tracing::info!("Successfully restored processes from YAML snapshot");

                // auto_start_on_restoreフラグが設定されたプロセスを自動起動
                match process_manager.start_auto_start_processes().await {
                    Ok(started) => {
                        if !started.is_empty() {
                            tracing::info!(
                                "Auto-started {} process(es): {:?}",
                                started.len(),
                                started
                            );
                        } else {
                            tracing::debug!("No processes marked for auto-start");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to auto-start processes: {}", e);
                        // 自動起動失敗はワーニングのみ、サーバー起動は継続
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to restore YAML snapshot: {}", e);
            }
        }
    } else {
        // YAMLスナップショットがない場合、レガシーインポートにフォールバック
        let import_file = env::var("ICHIMI_IMPORT_FILE").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(".ichimi")
                .join("snapshot.yaml")
                .to_string_lossy()
                .to_string()
        });

        if std::path::Path::new(&import_file).exists() {
            tracing::info!("Auto-importing processes from: {}", import_file);
            match process_manager.import_processes(&import_file).await {
                Ok(_) => {
                    tracing::info!("Successfully imported processes from {}", import_file);
                }
                Err(e) => {
                    tracing::warn!("Failed to auto-import processes: {}", e);
                }
            }
        } else {
            tracing::debug!("No import file found at: {}", import_file);
        }
    }

    // 注記: クリーンな状態を確保するため、シャットダウン時は常に全プロセスを停止します
    // プロセスは次回起動時にauto_start_on_restoreフラグに基づいて再起動されます
    tracing::info!("All processes will be stopped on shutdown for clean state management");

    // グレースフルシャットダウンのためのシグナルハンドラーをセットアップ
    let pm_for_shutdown = process_manager.clone();
    tokio::spawn(async move {
        let browser_proc = browser_process_for_shutdown;
        // SIGINT (Ctrl+C)とSIGTERMの両方を処理
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT (Ctrl+C), exporting processes and stopping all...");
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, exporting processes and stopping all...");
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = signal::ctrl_c().await;
            tracing::info!("Received shutdown signal, exporting processes and stopping all...");
        }

        // まず、自動起動プロセスのYAMLスナップショットを作成
        match pm_for_shutdown.create_auto_start_snapshot().await {
            Ok(path) => {
                tracing::info!("Created auto-start snapshot at {}", path);
            }
            Err(e) => {
                tracing::error!("Failed to create auto-start snapshot: {}", e);
            }
        }

        // 完全なYAMLスナップショットもエクスポート
        let export_file = env::var("ICHIMI_EXPORT_FILE").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(".ichimi")
                .join("snapshot.yaml")
                .to_string_lossy()
                .to_string()
        });

        // ディレクトリが存在しない場合は作成
        if let Some(parent) = std::path::Path::new(&export_file).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match pm_for_shutdown
            .export_processes(Some(export_file.clone()))
            .await
        {
            Ok(_) => tracing::info!("Successfully exported processes to {}", export_file),
            Err(e) => tracing::error!("Failed to export processes on shutdown: {}", e),
        }

        // 次にクリーンシャットダウンのため全プロセスを停止
        match pm_for_shutdown.stop_all_processes().await {
            Ok(stopped) => {
                if !stopped.is_empty() {
                    tracing::info!(
                        "Stopped {} process(es) for clean shutdown: {:?}",
                        stopped.len(),
                        stopped
                    );
                } else {
                    tracing::info!("No running processes to stop");
                }
            }
            Err(e) => {
                tracing::error!("Failed to stop processes: {}", e);
            }
        }

        // アプリモードで開かれていた場合、ブラウザを閉じる
        let mut browser_guard = browser_proc.lock().await;
        if let Some(mut child) = browser_guard.take() {
            let pid = child.id();
            tracing::info!("Closing browser window (PID: {})", pid);

            // プラットフォーム固有のグレースフルシャットダウン
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                // まずグレースフルシャットダウンのためSIGTERMを試行
                if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    tracing::debug!("Failed to send SIGTERM to browser: {}", e);
                } else {
                    // ブラウザがグレースフルに閉じる時間を与える
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        BROWSER_SHUTDOWN_GRACE_MS,
                    ))
                    .await;
                }

                // プロセスがまだ実行中かチェック
                match child.try_wait() {
                    Ok(Some(status)) => {
                        tracing::debug!("Browser closed gracefully with status: {:?}", status);
                    }
                    Ok(None) => {
                        // プロセスがまだ実行中、強制終了
                        tracing::debug!("Browser didn't close gracefully, forcing shutdown");
                        if let Err(e) = child.kill() {
                            tracing::warn!("Failed to force kill browser: {}", e);
                        } else {
                            let _ = child.wait();
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to check browser status: {}", e);
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // Windowsでは、より適切なkill()を使用
                if let Err(e) = child.kill() {
                    tracing::warn!("Failed to close browser window: {}", e);
                } else {
                    let _ = child.wait();
                }
            }
        }

        std::process::exit(0);
    });

    // 有効化されている場合、Webサーバーを起動
    #[cfg(feature = "web")]
    if web_enabled {
        tracing::info!("Web dashboard enabled on port {}", web_port);

        let web_manager = process_manager.clone();
        let web_persistence = process_manager.persistence_manager();

        // Webサーバーを起動し、実際のポートを取得
        let actual_port = match ichimi::web::start_web_server(
            web_manager,
            web_persistence,
            web_port,
        )
        .await
        {
            Ok(port) => {
                tracing::debug!("Web server started on actual port {}", port);
                port
            }
            Err(e) => {
                tracing::error!("Failed to start web server: {:?}", e);
                web_port // リクエストされたポートにフォールバック
            }
        };

        // 実際のポートでブラウザを開く（webが有効の場合）
        if auto_open && (web_enabled || web_only) {
            // Webダッシュボードが利用可能な場合、ブラウザを開く
            let url = format!("http://localhost:{actual_port}");

            let browser_proc = browser_process.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(BROWSER_STARTUP_DELAY_MS))
                    .await;

                if app_mode {
                    // アプリモード（専用ウィンドウ）でブラウザを開こうと試行
                    let browser_result = if cfg!(target_os = "macos") {
                        // macOS: まずChromeを試行、次にSafari
                        std::process::Command::new(
                            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                        )
                        .arg(format!("--app={}", url))
                        .arg("--new-window")
                        .spawn()
                        .or_else(|_| {
                            // Safariでopenコマンドにフォールバック
                            std::process::Command::new("open")
                                .arg("-n") // 新規インスタンス
                                .arg("-a")
                                .arg("Safari")
                                .arg(&url)
                                .spawn()
                        })
                    } else if cfg!(target_os = "windows") {
                        // Windows: Chromeを試行、次にEdge
                        std::process::Command::new("cmd")
                            .args(&["/C", "start", "chrome", &format!("--app={}", url)])
                            .spawn()
                            .or_else(|_| {
                                std::process::Command::new("cmd")
                                    .args(&["/C", "start", "msedge", &format!("--app={}", url)])
                                    .spawn()
                            })
                    } else {
                        // Linux: chromiumまたはgoogle-chromeを試行
                        std::process::Command::new("chromium")
                            .arg(format!("--app={}", url))
                            .spawn()
                            .or_else(|_| {
                                std::process::Command::new("google-chrome")
                                    .arg(format!("--app={}", url))
                                    .spawn()
                            })
                    };

                    match browser_result {
                        Ok(child) => {
                            tracing::info!(
                                "Opened browser in app mode at {} (PID: {:?})",
                                url,
                                child.id()
                            );
                            let mut browser_guard = browser_proc.lock().await;
                            *browser_guard = Some(child);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to open browser in app mode: {}. Falling back to normal mode.",
                                e
                            );
                            // 通常のブラウザ起動にフォールバック
                            if let Err(e) = open::that(&url) {
                                tracing::warn!("Failed to open browser: {}", e);
                            } else {
                                tracing::info!("Opening browser at {}", url);
                            }
                        }
                    }
                } else {
                    // 通常のブラウザ起動（既存の動作）
                    if let Err(e) = open::that(&url) {
                        tracing::warn!("Failed to open browser: {}", e);
                    } else {
                        tracing::info!("Opening browser at {}", url);
                    }
                }
            });
        }
    }

    // --web-onlyが指定されていない限り、MCPサーバーを実行
    if run_mcp {
        tracing::info!("Starting MCP server");
        let server = IchimiServer::with_process_manager(process_manager.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize IchimiServer: {}", e))?;
        let server_arc = std::sync::Arc::new(server);

        tracing::debug!("Serving MCP on stdio");
        match (*server_arc).clone().serve(stdio()).await {
            Ok(service) => {
                tracing::info!("MCP server ready, waiting for requests");
                service.waiting().await?;
                tracing::info!("MCP server shutting down");

                // MCPサーバー終了時も全プロセスを停止
                match process_manager.stop_all_processes().await {
                    Ok(stopped) => {
                        if !stopped.is_empty() {
                            tracing::info!(
                                "Stopped {} process(es) on MCP shutdown: {:?}",
                                stopped.len(),
                                stopped
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to stop processes on MCP shutdown: {}", e);
                    }
                }

                (*server_arc).shutdown().await.ok();
            }
            Err(e) => {
                tracing::warn!(
                    "MCP Server not available: {:?}. Web server will continue running.",
                    e
                );
                // Webサーバーのためプロセスを維持
                // シグナルハンドラーは上記で既にセットアップ済み、永久に待機
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(KEEPALIVE_INTERVAL_SECS))
                        .await;
                }
            }
        }
    } else {
        tracing::info!("Running in standalone mode (web server only)");
        // プロセスを維持 - スポーンされたタスクのシグナルハンドラーがシャットダウンを処理
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(KEEPALIVE_INTERVAL_SECS)).await;
        }
    }

    #[cfg(not(feature = "web"))]
    if web_enabled {
        tracing::warn!("Web feature not enabled. Rebuild with --features web to enable dashboard.");
    }

    tracing::info!("Ichimi server shutdown complete");
    Ok(())
}
