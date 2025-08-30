use anyhow::Result;
use ichimi_server::IchimiServer;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use tokio::signal;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut web_enabled = cfg!(feature = "web"); // Default to true if web feature is enabled
    let mut web_port = 12700u16;
    let mut auto_open = true; // Default to auto-open browser

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("Ichimi Server - Process management server for Claude Code via MCP");
                println!();
                println!("Usage: ichimi [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --help, -h       Show this help message");
                println!("  --version, -v    Show version information");
                println!("  --web            Enable web dashboard (default: enabled)");
                println!("  --no-web         Disable web dashboard");
                println!("  --web-port PORT  Set web dashboard port (default: 12700)");
                println!("  --no-open        Don't automatically open browser for web dashboard");
                return Ok(());
            }
            "--version" | "-v" => {
                println!("ichimi-server v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--web" => {
                web_enabled = true;
            }
            "--no-web" => {
                web_enabled = false;
            }
            "--web-port" => {
                if i + 1 < args.len() {
                    web_port = args[i + 1].parse().unwrap_or(12700);
                    i += 1;
                }
            }
            "--no-open" => {
                auto_open = false;
            }
            _ => {}
        }
        i += 1;
    }

    // Detect if running as MCP server
    let is_mcp = env::var("MCP_SERVER_NAME").is_ok() || env::var("CLAUDE_CODE").is_ok();

    // Setup logging based on environment
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    if is_mcp {
        // When running as MCP, log to file to avoid interfering with stdio
        let log_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".ichimi")
            .join("logs");

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&log_dir).ok();

        // Generate log filename with timestamp
        let log_file = log_dir.join(format!(
            "ichimi-mcp-{}.log",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        ));

        // Create file appender
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .expect("Failed to create log file");

        let filter = EnvFilter::from_default_env()
            .add_directive(format!("ichimi={log_level}").parse().unwrap())
            .add_directive(format!("ichimi_server={log_level}").parse().unwrap())
            .add_directive("facet_kdl=warn".parse().unwrap())
            .add_directive("mcp_server=debug".parse().unwrap());

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();

        tracing::info!("=== Ichimi MCP Server Starting ===");
        tracing::info!("Log file: {:?}", log_file);
        tracing::info!(
            "Environment: MCP_SERVER_NAME={:?}",
            env::var("MCP_SERVER_NAME").ok()
        );
        tracing::info!("Arguments: {:?}", args);
        tracing::info!("Working directory: {:?}", env::current_dir());

        // Also write startup info to stderr for debugging
        eprintln!("[ICHIMI] MCP mode detected, logging to: {log_file:?}");
    } else {
        // Normal mode - log to stderr
        let filter = EnvFilter::from_default_env()
            .add_directive(format!("ichimi={log_level}").parse().unwrap())
            .add_directive(format!("ichimi_server={log_level}").parse().unwrap())
            .add_directive("facet_kdl=warn".parse().unwrap());

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();

        tracing::info!("Starting Ichimi Server (console mode)");
    }

    // Create a shared process manager
    let process_manager = ichimi_server::process::ProcessManager::new().await;

    // Auto-import processes on startup if configured
    let import_file = env::var("ICHIMI_IMPORT_FILE").unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".ichimi")
            .join("data")
            .join("processes.surql")
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

    // Setup signal handler for graceful shutdown
    let pm_for_shutdown = process_manager.clone();
    tokio::spawn(async move {
        let _ = signal::ctrl_c().await;
        tracing::info!("Received shutdown signal, exporting processes...");

        // Auto-export processes on shutdown
        let export_file = env::var("ICHIMI_EXPORT_FILE").unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".ichimi")
                .join("data")
                .join("processes.surql")
                .to_string_lossy()
                .to_string()
        });

        // Create directory if it doesn't exist
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

        std::process::exit(0);
    });

    // Start web server if enabled
    #[cfg(feature = "web")]
    if web_enabled {
        tracing::info!("Web dashboard enabled on port {}", web_port);

        let web_manager = process_manager.clone();
        let web_port_clone = web_port;

        // Spawn web server in background
        tokio::spawn(async move {
            tracing::debug!("Starting web server in background");
            if let Err(e) = ichimi_server::web::start_web_server(web_manager, web_port_clone).await
            {
                tracing::error!("Web server error: {:?}", e);
            }
        });

        // Open browser after a short delay to allow server to start
        if auto_open && !is_mcp {
            // Don't open browser in MCP mode
            let url = format!("http://localhost:{web_port}");
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if let Err(e) = open::that(&url) {
                    tracing::warn!("Failed to open browser: {}", e);
                } else {
                    tracing::info!("Opening browser at {}", url);
                }
            });
        }
    }

    // Run MCP server if possible
    if is_mcp || isatty::stderr_isatty() {
        tracing::info!("Starting MCP server");
        let server = IchimiServer::with_process_manager(process_manager.clone()).await;
        let server_arc = std::sync::Arc::new(server);

        tracing::debug!("Serving MCP on stdio");
        match (*server_arc).clone().serve(stdio()).await {
            Ok(service) => {
                tracing::info!("MCP server ready, waiting for requests");
                service.waiting().await?;
                tracing::info!("MCP server shutting down");
                (*server_arc).shutdown().await.ok();
            }
            Err(e) => {
                tracing::warn!(
                    "MCP Server not available: {:?}. Web server will continue running.",
                    e
                );
                // Keep the process alive for web server
                // Signal handler already set up above, just wait forever
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                }
            }
        }
    } else {
        tracing::info!("Running in standalone mode (web server only)");
        // Keep the process alive for web server
        // Signal handler already set up above, just wait forever
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    #[cfg(not(feature = "web"))]
    if web_enabled {
        tracing::warn!("Web feature not enabled. Rebuild with --features web to enable dashboard.");
    }

    tracing::info!("Ichimi server shutdown complete");
    Ok(())
}
