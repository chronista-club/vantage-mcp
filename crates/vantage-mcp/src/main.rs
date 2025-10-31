use anyhow::Result;
use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use tracing_subscriber::{self, EnvFilter};
use vantage::VantageServer;

// メンテナビリティ向上のための定数
const BROWSER_STARTUP_DELAY_MS: u64 = 500;
const KEEPALIVE_INTERVAL_SECS: u64 = 3600;

/// Vantage MCP - MCP経由のClaude Code用プロセス管理サーバー
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Webダッシュボード用のブラウザを自動的に開かない
    #[arg(long)]
    no_open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // clapを使用してコマンドライン引数をパース
    let cli = Cli::parse();

    // CLI引数から設定を導出
    let auto_open = !cli.no_open;
    let web_port = 12700; // デフォルトポート（衝突時は自動変更）

    // 環境に基づいてロギングをセットアップ
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // Webモード有効のため、stderrにログ出力
    let filter = EnvFilter::from_default_env()
        .add_directive(
            format!("vantage={log_level}")
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?,
        )
        .add_directive(
            format!("vantage_mcp={log_level}")
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

    tracing::info!("Starting Vantage MCP (MCP + Web mode)");

    // 共有プロセスマネージャーを作成
    let process_manager = vantage::atom::process::ProcessManager::new().await;

    // 設定されている場合、起動時にプロセスを自動インポート
    // まず自動起動プロセス用のYAMLスナップショットを試行
    let yaml_snapshot = std::env::var("HOME")
        .map(|home| format!("{home}/.vantage/snapshot.yaml"))
        .unwrap_or_else(|_| ".vantage/snapshot.yaml".to_string());

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
        let import_file = env::var("VANTAGE_IMPORT_FILE").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(".vantage")
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
        let export_file = env::var("VANTAGE_EXPORT_FILE").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(".vantage")
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

        std::process::exit(0);
    });

    // Webサーバーを起動
    tracing::info!("Web dashboard enabled on port {}", web_port);

    let web_manager = process_manager.clone();
    let web_persistence = process_manager.persistence_manager();

    // Webサーバーを起動し、実際のポートを取得
    let actual_port =
        match vantage::atom::web::start_web_server(web_manager, web_persistence, web_port).await {
            Ok(port) => {
                tracing::debug!("Web server started on actual port {}", port);
                port
            }
            Err(e) => {
                tracing::error!("Failed to start web server: {:?}", e);
                web_port // リクエストされたポートにフォールバック
            }
        };

    // 実際のポートでブラウザを開く
    if auto_open {
        let url = format!("http://localhost:{actual_port}");
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(BROWSER_STARTUP_DELAY_MS))
                .await;

            if let Err(e) = open::that(&url) {
                tracing::warn!("Failed to open browser: {}", e);
            } else {
                tracing::info!("Opening browser at {}", url);
            }
        });
    }

    // MCPサーバーを起動
    tracing::info!("Starting MCP server");
    let server = VantageServer::with_process_manager(process_manager.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize VantageServer: {}", e))?;
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
                tokio::time::sleep(tokio::time::Duration::from_secs(KEEPALIVE_INTERVAL_SECS)).await;
            }
        }
    }

    tracing::info!("Vantage MCP shutdown complete");
    Ok(())
}
