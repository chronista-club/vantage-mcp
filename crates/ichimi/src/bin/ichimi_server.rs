use anyhow::Result;
use ichimi_server::{IchimiServer, process::ProcessManager};
use rmcp::{ServiceExt, model::ServerInfo};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Constants for better maintainability
const DEFAULT_WEB_PORT: u16 = 12700;
const BROWSER_STARTUP_DELAY_MS: u64 = 500;
const BROWSER_SHUTDOWN_GRACE_MS: u64 = 1000;
const KEEPALIVE_INTERVAL_SECS: u64 = 3600;

#[derive(Debug, Clone)]
enum DefaultBrowser {
    Chrome,
    Firefox,
    Safari,
    Unknown,
}

// Detect the default browser on the system
fn detect_default_browser() -> DefaultBrowser {
    #[cfg(target_os = "macos")]
    {
        // Try to read macOS default browser preference
        let output = std::process::Command::new("plutil")
            .args(["-p", &format!("{}/Library/Preferences/com.apple.LaunchServices/com.apple.launchservices.secure.plist", env::var("HOME").unwrap_or_default())])
            .output();

        if output.is_err() {
            // Try alternate location
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
        // Check Windows registry for default browser
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
        // Check xdg-settings for default browser
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
    // Initialize tracing with environment-based log level
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(rust_log)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .init();

    tracing::info!("Starting Ichimi Server v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments
    let mut args = std::env::args();
    let _ = args.next(); // Skip program name
    let mut enable_web = false;
    let mut web_port = DEFAULT_WEB_PORT;
    let mut web_only = false;
    let mut no_web = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("Ichimi Server v{}", env!("CARGO_PKG_VERSION"));
                println!("\nUsage: ichimi [OPTIONS]");
                println!("\nOptions:");
                println!("  --web              Enable web interface");
                println!("  --web-port <PORT>  Web interface port (default: {})", DEFAULT_WEB_PORT);
                println!("  --web-only         Run only the web interface (no MCP server)");
                println!("  --no-web           Disable web interface completely");
                println!("  --help, -h         Show this help message");
                println!("\nEnvironment variables:");
                println!("  RUST_LOG                        Log level (error, warn, info, debug, trace)");
                println!("  ICHIMI_AUTO_EXPORT_INTERVAL    Auto-export interval in seconds");
                println!("  ICHIMI_IMPORT_FILE              File to import on startup");
                println!("  ICHIMI_EXPORT_FILE              File to export on shutdown");
                return Ok(());
            }
            "--web" => {
                enable_web = true;
                tracing::info!("Web interface enabled");
            }
            "--web-port" => {
                if let Some(port_str) = args.next() {
                    web_port = port_str.parse().unwrap_or_else(|_| {
                        tracing::warn!("Invalid port '{}', using default {}", port_str, DEFAULT_WEB_PORT);
                        DEFAULT_WEB_PORT
                    });
                }
            }
            "--web-only" => {
                web_only = true;
                enable_web = true;
                tracing::info!("Running in web-only mode (no MCP server)");
            }
            "--no-web" => {
                no_web = true;
                tracing::info!("Web interface disabled");
            }
            _ => {
                tracing::warn!("Unknown argument: {}", arg);
            }
        }
    }

    // Validate conflicting options
    if web_only && no_web {
        eprintln!("Error: Cannot use both --web-only and --no-web");
        std::process::exit(1);
    }

    // Check for deprecated environment variable
    if env::var("ICHIMI_ENABLE_WEB").is_ok() {
        tracing::warn!("ICHIMI_ENABLE_WEB is deprecated. Use --web flag instead.");
    }

    let server_info = ServerInfo::default();

    // Initialize shared process manager
    let process_manager = ProcessManager::new().await;

    // Create process for browser if --web-only is used (for shutdown handling)
    let browser_process: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let browser_process_for_shutdown = browser_process.clone();

    // Create WebDriver client holder
    #[cfg(feature = "webdriver")]
    let webdriver_client: Arc<Mutex<Option<fantoccini::Client>>> = Arc::new(Mutex::new(None));
    #[cfg(feature = "webdriver")]
    let webdriver_client_for_shutdown = webdriver_client.clone();

    // Setup and run web interface if enabled (either through --web or --web-only)
    let web_handle = if enable_web && !no_web {
        let pm_clone = process_manager.clone();
        let port = web_port;

        Some(tokio::spawn(async move {
            // Read web dist directory path from environment or use default
            let dist_dir = env::var("ICHIMI_WEB_DIST").unwrap_or_else(|_| {
                // Try to find the dist directory relative to the binary
                let exe_path = std::env::current_exe().expect("Failed to get executable path");
                
                // Look for the dist directory in several possible locations
                let possible_paths = vec![
                    // For development: relative to cargo workspace root
                    exe_path.parent().unwrap().parent().unwrap().parent().unwrap().join("ui/web/dist"),
                    // For installed binary: in the same directory
                    exe_path.parent().unwrap().join("ui/web/dist"),
                    // Current directory fallback
                    std::path::PathBuf::from("ui/web/dist"),
                ];
                
                for path in &possible_paths {
                    if path.exists() {
                        tracing::info!("Found web dist directory at: {}", path.display());
                        return path.to_string_lossy().to_string();
                    }
                }
                
                // Default fallback
                let default = "ui/web/dist".to_string();
                tracing::warn!("Web dist directory not found in expected locations, using: {}", default);
                default
            });

            // Check if dist directory exists
            if !std::path::Path::new(&dist_dir).exists() {
                tracing::error!("Web dist directory not found at: {}. Please build the web UI with 'bun run build' in ui/web-vue/", dist_dir);
                return;
            }

            // Start the web server with the process manager
            let persistence_manager = pm_clone.persistence_manager();
            match ichimi_server::web::start_web_server(pm_clone, persistence_manager, port).await {
                Ok(actual_port) => {
                    tracing::info!("Web server started on port {}", actual_port);
                }
                Err(e) => {
                    tracing::error!("Failed to start web server: {}", e);
                }
            }
        }))
    } else {
        None
    };

    // Handle web-only mode
    if web_only {
        #[cfg(feature = "webdriver")]
        {
            // Import webdriver features
            use ichimi::webdriver::{WebDriverManager, BrowserType};
            
            // Try to launch browser automatically after a short delay
            let web_port_for_browser = web_port;
            let browser_proc = browser_process.clone();
            let webdriver_client_clone = webdriver_client.clone();
            tokio::spawn(async move {
                // Wait for the web server to be ready
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                
                let url = format!("http://localhost:{}", web_port_for_browser);
                
                // Try to use WebDriver first for better control
                match WebDriverManager::new().await {
                    Ok(manager) => {
                        // Try browsers in order of preference
                        let browsers_to_try = vec![
                            BrowserType::Chrome,
                            BrowserType::Firefox,
                            BrowserType::Safari,
                            BrowserType::Edge,
                        ];
                        
                        for browser_type in browsers_to_try {
                            match manager.launch_browser(&url, browser_type).await {
                                Ok(client) => {
                                    tracing::info!("Successfully opened {} browser at {}", browser_type, url);
                                    *webdriver_client_clone.lock().await = Some(client);
                                    return;
                                }
                                Err(e) => {
                                    tracing::debug!("Failed to open {} browser: {}", browser_type, e);
                                }
                            }
                        }
                        tracing::warn!("Could not open any browser via WebDriver, falling back to system command");
                    }
                    Err(e) => {
                        tracing::debug!("WebDriver not available: {}, falling back to system command", e);
                    }
                }
                
                // Fallback to system command
                let result = if cfg!(target_os = "macos") {
                    std::process::Command::new("open")
                        .arg(url.clone())
                        .spawn()
                } else if cfg!(target_os = "linux") {
                    std::process::Command::new("xdg-open")
                        .arg(url.clone())
                        .spawn()
                } else if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd")
                        .args(&["/c", "start", "", &url])
                        .spawn()
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Unsupported OS"
                    ))
                };
                
                match result {
                    Ok(child) => {
                        tracing::info!("Opened browser at {}", url);
                        *browser_proc.lock().await = Some(child);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to open browser: {}", e);
                        tracing::info!("Please open your browser and navigate to {}", url);
                    }
                }
            });
        }
        
        #[cfg(not(feature = "webdriver"))]
        {
            // Simple browser launch without WebDriver
            let web_port_for_browser = web_port;
            let browser_proc = browser_process.clone();
            tokio::spawn(async move {
                // Wait for the web server to be ready
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                
                let url = format!("http://localhost:{}", web_port_for_browser);
                
                // Try to open browser
                let result = if cfg!(target_os = "macos") {
                    std::process::Command::new("open")
                        .arg(url.clone())
                        .spawn()
                } else if cfg!(target_os = "linux") {
                    std::process::Command::new("xdg-open")
                        .arg(url.clone())
                        .spawn()
                } else if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd")
                        .args(&["/c", "start", "", &url])
                        .spawn()
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Unsupported OS"
                    ))
                };
                
                match result {
                    Ok(child) => {
                        tracing::info!("Opened browser at {}", url);
                        *browser_proc.lock().await = Some(child);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to open browser: {}", e);
                        tracing::info!("Please open your browser and navigate to {}", url);
                    }
                }
            });
        }
        
        tracing::info!("Web interface is running at http://localhost:{}", web_port);
    }

    // Import processes from file if specified
    let import_file = env::var("ICHIMI_IMPORT_FILE").unwrap_or_else(|_| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".ichimi")
            .join("auto_start.yaml")
            .to_string_lossy()
            .to_string()
    });

    if std::path::Path::new(&import_file).exists() {
        tracing::info!("Importing processes from: {}", import_file);
        match process_manager.import_processes(&import_file).await {
            Ok(_) => tracing::info!("Successfully imported processes from {}", import_file),
            Err(e) => tracing::error!("Failed to import processes from {}: {}", import_file, e),
        }
    } else {
        tracing::debug!("No import file found at: {}", import_file);
    }

    // グレースフルシャットダウン: 全プロセスを適切に停止
    // auto_start_on_restoreフラグに基づいて次回起動時に自動再開
    tracing::info!("Graceful shutdown enabled: all processes will be stopped with 5s grace period");

    // Setup signal handler for graceful shutdown
    let pm_for_shutdown = process_manager.clone();
    tokio::spawn(async move {
        let browser_proc = browser_process_for_shutdown;
        #[cfg(feature = "webdriver")]
        let webdriver_client = webdriver_client_for_shutdown;
        // Handle both SIGINT (Ctrl+C) and SIGTERM
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigint =
                signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");

            tokio::select! {
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT (Ctrl+C), performing graceful shutdown...");
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, performing graceful shutdown...");
                }
            }
        }

        #[cfg(not(unix))]
        {
            let _ = signal::ctrl_c().await;
            tracing::info!("Received shutdown signal, performing graceful shutdown...");
        }

        // First, create YAML snapshot of auto-start processes
        match pm_for_shutdown.create_auto_start_snapshot().await {
            Ok(path) => {
                tracing::info!("Created auto-start snapshot at {}", path);
            }
            Err(e) => {
                tracing::error!("Failed to create auto-start snapshot: {}", e);
            }
        }

        // Also export the full YAML snapshot
        let export_file = env::var("ICHIMI_EXPORT_FILE").unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join(".ichimi")
                .join("snapshot.yaml")
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

        // グレースフルシャットダウン: 全プロセスを5秒の猶予期間で停止
        tracing::info!("Starting graceful shutdown of all managed processes...");
        match pm_for_shutdown.stop_all_processes().await {
            Ok(stopped) => {
                if !stopped.is_empty() {
                    tracing::info!(
                        "Successfully stopped {} process(es) with graceful shutdown: {:?}",
                        stopped.len(),
                        stopped
                    );
                } else {
                    tracing::info!("No running processes to stop");
                }
            }
            Err(e) => {
                tracing::error!("Failed to stop processes gracefully: {}", e);
            }
        }

        // Close WebDriver browser if it was used
        #[cfg(feature = "webdriver")]
        {
            let mut client_guard = webdriver_client.lock().await;
            if let Some(client) = client_guard.take() {
                tracing::info!("Closing WebDriver browser session");
                if let Err(e) = client.close().await {
                    tracing::warn!("Failed to close WebDriver browser: {}", e);
                }
            }
        }
        
        // Close browser process if it was opened
        let mut browser_guard = browser_proc.lock().await;
        if let Some(mut child) = browser_guard.take() {
            tracing::info!("Closing browser process");
            
            // Get the process ID for platform-specific handling
            let pid = child.id();
            // Platform-specific graceful shutdown
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                // First try SIGTERM for graceful shutdown
                if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    tracing::debug!("Failed to send SIGTERM to browser: {}", e);
                } else {
                    // Give it a moment to close gracefully
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }

            // Check if still running and force kill if needed
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::debug!("Browser process exited with: {:?}", status);
                }
                Ok(None) => {
                    // Still running, force kill
                    if let Err(e) = child.kill() {
                        tracing::warn!("Failed to kill browser process: {}", e);
                    } else {
                        tracing::debug!("Browser didn't close gracefully, forcing shutdown");
                        let _ = child.wait();
                    }
                }
                Err(e) => {
                    tracing::warn!("Error checking browser process status: {}", e);
                }
            }
        }

        // Signal shutdown complete
        std::process::exit(0);
    });

    // Skip MCP server if in web-only mode
    if web_only {
        tracing::info!("Running in web-only mode, MCP server is disabled");
        
        // Wait for the web server task
        if let Some(handle) = web_handle {
            let _ = handle.await;
        }
        
        return Ok(());
    }

    // Create the server
    let server = IchimiServer::with_process_manager(process_manager.clone()).await?;

    // Setup auto-export if interval is specified
    let auto_export_interval = env::var("ICHIMI_AUTO_EXPORT_INTERVAL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok());

    if let Some(interval_secs) = auto_export_interval {
        if interval_secs > 0 {
            let pm_clone = process_manager.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
                interval.tick().await; // Skip first immediate tick

                let export_dir = env::var("ICHIMI_DATA_DIR").unwrap_or_else(|_| {
                    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    format!("{home}/.ichimi/data")
                });

                // Ensure the export directory exists
                if let Err(e) = tokio::fs::create_dir_all(&export_dir).await {
                    tracing::error!("Failed to create export directory {}: {}", export_dir, e);
                    return;
                }

                let export_file = format!("{export_dir}/processes.surql");

                loop {
                    interval.tick().await;
                    tracing::debug!("Auto-exporting processes to {}", export_file);

                    match pm_clone.export_processes(Some(export_file.clone())).await {
                        Ok(_) => {
                            tracing::info!("Auto-exported processes to {}", export_file);
                        }
                        Err(e) => {
                            tracing::error!("Auto-export failed: {}", e);
                        }
                    }
                }
            });

            tracing::info!(
                "Auto-export enabled: processes will be exported every {} seconds",
                interval_secs
            );
        }
    }

    // Create MCP transport
    let transport = (
        tokio::io::stdin(),
        tokio::io::stdout(),
    );

    // Run the MCP server using serve method
    tracing::info!("MCP server is ready, waiting for connections...");
    let server_handle = server.serve(transport).await?;
    
    // Wait for server shutdown
    let quit_reason = server_handle.waiting().await?;
    tracing::info!("MCP server shutting down: {:?}", quit_reason);

    // Perform cleanup on MCP shutdown
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

    Ok(())
}
