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
    let mut web_enabled = false; // Web is disabled by default (MCP mode is default)
    let mut web_only = false; // Flag to run only web server without MCP
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
                println!("Default: Run as MCP server for Claude/Cline");
                println!();
                println!("Options:");
                println!("  --help, -h       Show this help message");
                println!("  --version, -v    Show version information");
                println!("  --web            Enable web dashboard alongside MCP server");
                println!("  --web-only       Run only web dashboard (no MCP server)");
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
            "--web-only" => {
                web_enabled = true;
                web_only = true;
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

    // Determine operation mode
    let run_mcp = !web_only;  // Run MCP server by default unless --web-only is specified
    
    // Setup logging based on environment
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // When running as MCP (default), log to file to avoid interfering with stdio
    if run_mcp && !web_enabled {
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
            .map_err(|e| anyhow::anyhow!("Failed to create log file: {}", e))?;

        let filter = EnvFilter::from_default_env()
            .add_directive(format!("ichimi={log_level}").parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?)
            .add_directive(format!("ichimi_server={log_level}").parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?)
            .add_directive("facet_kdl=warn".parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?)
            .add_directive("mcp_server=debug".parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?);

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
        // Web mode or MCP with web - log to stderr
        let filter = EnvFilter::from_default_env()
            .add_directive(format!("ichimi={log_level}").parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?)
            .add_directive(format!("ichimi_server={log_level}").parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?)
            .add_directive("facet_kdl=warn".parse().map_err(|e| anyhow::anyhow!("Invalid log level: {}", e))?);

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();

        if web_only {
            tracing::info!("Starting Ichimi Server (web-only mode)");
        } else if run_mcp && web_enabled {
            tracing::info!("Starting Ichimi Server (MCP + web mode)");
        } else {
            tracing::info!("Starting Ichimi Server (MCP mode)");
        }
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

        // Start web server and get actual port
        let actual_port = match ichimi_server::web::start_web_server(web_manager, web_port).await {
            Ok(port) => {
                tracing::debug!("Web server started on actual port {}", port);
                port
            }
            Err(e) => {
                tracing::error!("Failed to start web server: {:?}", e);
                web_port // Fall back to requested port
            }
        };

        // Open browser with actual port (only in web-only mode)
        if auto_open && web_only {
            // Don't open browser in MCP mode
            let url = format!("http://localhost:{}", actual_port);
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

    // Run MCP server unless --web-only is specified
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
