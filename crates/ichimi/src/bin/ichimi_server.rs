use anyhow::Result;
use ichimi_server::IchimiServer;
use rmcp::{ServiceExt, transport::stdio};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{self, EnvFilter};

// Constants for better maintainability
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
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut web_enabled = false; // Web is disabled by default (MCP mode is default)
    let mut web_only = false; // Flag to run only web server without MCP
    let mut web_port = 12700u16;
    let mut auto_open = true; // Default to auto-open browser
    let mut app_mode = false; // Open browser in app mode
    #[cfg(feature = "webdriver")]
    let mut use_webdriver = false; // Use WebDriver for browser control

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
                println!(
                    "  --app-mode       Open browser in app mode (dedicated window that closes with server)"
                );
                #[cfg(feature = "webdriver")]
                println!(
                    "  --webdriver      Use WebDriver to control browser tabs (requires geckodriver/chromedriver)"
                );
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
                    match args[i + 1].parse::<u16>() {
                        Ok(port) if port > 0 => web_port = port,
                        Ok(_) => {
                            eprintln!(
                                "Warning: Port must be between 1 and 65535, using default {}",
                                12700
                            );
                        }
                        Err(_) => {
                            eprintln!(
                                "Warning: Invalid port '{}', using default {}",
                                args[i + 1],
                                12700
                            );
                        }
                    }
                    i += 1;
                }
            }
            "--no-open" => {
                auto_open = false;
            }
            "--app-mode" => {
                app_mode = true;
            }
            #[cfg(feature = "webdriver")]
            "--webdriver" => {
                use_webdriver = true;
                app_mode = false; // WebDriver and app-mode are mutually exclusive
            }
            _ => {}
        }
        i += 1;
    }

    // Determine operation mode
    let run_mcp = !web_only; // Run MCP server by default unless --web-only is specified

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
        // Web mode or MCP with web - log to stderr
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

    // Create a shared process manager
    let process_manager = ichimi_server::process::ProcessManager::new().await;

    // Track browser process for cleanup
    let browser_process: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let browser_process_for_shutdown = browser_process.clone();

    // Track WebDriver client for cleanup
    #[cfg(feature = "webdriver")]
    let webdriver_client: Arc<Mutex<Option<fantoccini::Client>>> = Arc::new(Mutex::new(None));
    #[cfg(feature = "webdriver")]
    let webdriver_client_for_shutdown = webdriver_client.clone();

    // Auto-import processes on startup if configured
    // First try YAML snapshot for auto-start processes
    let yaml_snapshot = std::env::var("HOME")
        .map(|home| format!("{home}/.ichimi/snapshot.yaml"))
        .unwrap_or_else(|_| ".ichimi/snapshot.yaml".to_string());

    if std::path::Path::new(&yaml_snapshot).exists() {
        tracing::info!("Restoring from YAML snapshot: {}", yaml_snapshot);
        match process_manager.restore_yaml_snapshot().await {
            Ok(_) => {
                tracing::info!("Successfully restored processes from YAML snapshot");
            }
            Err(e) => {
                tracing::warn!("Failed to restore YAML snapshot: {}", e);
            }
        }
    } else {
        // Fall back to legacy import if no YAML snapshot
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

    // Note: We always stop all processes on shutdown to ensure clean state
    // Processes will be restarted on next launch based on auto_start_on_restore flag
    tracing::info!("All processes will be stopped on shutdown for clean state management");

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

        // Then stop ALL processes for clean shutdown
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

        // Close browser if it was opened in app mode
        let mut browser_guard = browser_proc.lock().await;
        if let Some(mut child) = browser_guard.take() {
            let pid = child.id();
            tracing::info!("Closing browser window (PID: {})", pid);

            // Platform-specific graceful shutdown
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                // First try SIGTERM for graceful shutdown
                if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    tracing::debug!("Failed to send SIGTERM to browser: {}", e);
                } else {
                    // Give the browser time to close gracefully
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        BROWSER_SHUTDOWN_GRACE_MS,
                    ))
                    .await;
                }

                // Check if process is still running
                match child.try_wait() {
                    Ok(Some(status)) => {
                        tracing::debug!("Browser closed gracefully with status: {:?}", status);
                    }
                    Ok(None) => {
                        // Process still running, force kill
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
                // On Windows, just use kill() which is more appropriate
                if let Err(e) = child.kill() {
                    tracing::warn!("Failed to close browser window: {}", e);
                } else {
                    let _ = child.wait();
                }
            }
        }

        std::process::exit(0);
    });

    // Start web server if enabled
    #[cfg(feature = "web")]
    if web_enabled {
        tracing::info!("Web dashboard enabled on port {}", web_port);

        let web_manager = process_manager.clone();
        let web_persistence = process_manager.persistence_manager();

        // Start web server and get actual port
        let actual_port = match ichimi_server::web::start_web_server(
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
                web_port // Fall back to requested port
            }
        };

        // Open browser with actual port (when web is enabled)
        if auto_open && (web_enabled || web_only) {
            // Open browser when web dashboard is available
            let url = format!("http://localhost:{actual_port}");

            #[cfg(feature = "webdriver")]
            if use_webdriver {
                // Use WebDriver to open browser
                let webdriver_client_clone = webdriver_client.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        BROWSER_STARTUP_DELAY_MS,
                    ))
                    .await;

                    // Detect default browser and try appropriate WebDriver
                    let default_browser = detect_default_browser();
                    tracing::info!("Detected default browser: {:?}", default_browser);

                    // Try to connect to WebDriver based on detected browser
                    let webdriver_result = async {
                        match default_browser {
                            DefaultBrowser::Chrome => {
                                // Try chromedriver first for Chrome
                                match fantoccini::ClientBuilder::native()
                                    .connect("http://localhost:9515")
                                    .await
                                {
                                    Ok(client) => Ok(client),
                                    Err(_) => {
                                        // Fallback to geckodriver
                                        fantoccini::ClientBuilder::native()
                                            .connect("http://localhost:4444")
                                            .await
                                    }
                                }
                            }
                            DefaultBrowser::Firefox => {
                                // Try geckodriver first for Firefox
                                match fantoccini::ClientBuilder::native()
                                    .connect("http://localhost:4444")
                                    .await
                                {
                                    Ok(client) => Ok(client),
                                    Err(_) => {
                                        // Fallback to chromedriver
                                        fantoccini::ClientBuilder::native()
                                            .connect("http://localhost:9515")
                                            .await
                                    }
                                }
                            }
                            DefaultBrowser::Safari => {
                                // Safari uses safaridriver on port 9515 by default
                                match fantoccini::ClientBuilder::native()
                                    .connect("http://localhost:9515")
                                    .await
                                {
                                    Ok(client) => Ok(client),
                                    Err(_) => {
                                        // Fallback to geckodriver
                                        fantoccini::ClientBuilder::native()
                                            .connect("http://localhost:4444")
                                            .await
                                    }
                                }
                            }
                            DefaultBrowser::Unknown => {
                                // Try both in order
                                match fantoccini::ClientBuilder::native()
                                    .connect("http://localhost:9515")
                                    .await
                                {
                                    Ok(client) => Ok(client),
                                    Err(_) => {
                                        fantoccini::ClientBuilder::native()
                                            .connect("http://localhost:4444")
                                            .await
                                    }
                                }
                            }
                        }
                    }
                    .await;

                    match webdriver_result {
                        Ok(client) => {
                            // Navigate to the URL
                            if let Err(e) = client.goto(&url).await {
                                tracing::warn!("Failed to navigate to {}: {}", url, e);
                            } else {
                                tracing::info!("Opened browser tab via WebDriver at {}", url);

                                // Store the client for later cleanup
                                let mut client_guard = webdriver_client_clone.lock().await;
                                *client_guard = Some(client);
                            }
                        }
                        Err(e) => {
                            let driver_hint = match default_browser {
                                DefaultBrowser::Chrome => "chromedriver (port 9515)",
                                DefaultBrowser::Firefox => "geckodriver (port 4444)",
                                DefaultBrowser::Safari => "safaridriver (port 9515)",
                                DefaultBrowser::Unknown => {
                                    "geckodriver (port 4444) or chromedriver (port 9515)"
                                }
                            };
                            tracing::warn!(
                                "Failed to connect to WebDriver: {}. Make sure {} is running.",
                                e,
                                driver_hint
                            );
                            // Fallback to normal browser open
                            if let Err(e) = open::that(&url) {
                                tracing::warn!("Failed to open browser: {}", e);
                            } else {
                                tracing::info!("Opened browser at {} (fallback)", url);
                            }
                        }
                    }
                });
            } else {
                // Non-WebDriver browser opening
                let browser_proc = browser_process.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        BROWSER_STARTUP_DELAY_MS,
                    ))
                    .await;

                    if app_mode {
                        // Try to open browser in app mode (dedicated window)
                        let browser_result = if cfg!(target_os = "macos") {
                            // macOS: Try Chrome first, then Safari
                            std::process::Command::new(
                                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                            )
                            .arg(format!("--app={url}"))
                            .arg("--new-window")
                            .spawn()
                            .or_else(|_| {
                                // Fallback to open command with Safari
                                std::process::Command::new("open")
                                    .arg("-n") // New instance
                                    .arg("-a")
                                    .arg("Safari")
                                    .arg(&url)
                                    .spawn()
                            })
                        } else if cfg!(target_os = "windows") {
                            // Windows: Try Chrome, then Edge
                            std::process::Command::new("cmd")
                                .args(["/C", "start", "chrome", &format!("--app={url}")])
                                .spawn()
                                .or_else(|_| {
                                    std::process::Command::new("cmd")
                                        .args(["/C", "start", "msedge", &format!("--app={url}")])
                                        .spawn()
                                })
                        } else {
                            // Linux: Try chromium or google-chrome
                            std::process::Command::new("chromium")
                                .arg(format!("--app={url}"))
                                .spawn()
                                .or_else(|_| {
                                    std::process::Command::new("google-chrome")
                                        .arg(format!("--app={url}"))
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
                                // Fallback to normal browser open
                                if let Err(e) = open::that(&url) {
                                    tracing::warn!("Failed to open browser: {}", e);
                                } else {
                                    tracing::info!("Opening browser at {}", url);
                                }
                            }
                        }
                    } else {
                        // Normal browser open (existing behavior)
                        if let Err(e) = open::that(&url) {
                            tracing::warn!("Failed to open browser: {}", e);
                        } else {
                            tracing::info!("Opening browser at {}", url);
                        }
                    }
                });
            }

            #[cfg(not(feature = "webdriver"))]
            {
                let browser_proc = browser_process.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        BROWSER_STARTUP_DELAY_MS,
                    ))
                    .await;

                    if app_mode {
                        // Try to open browser in app mode (dedicated window)
                        let browser_result = if cfg!(target_os = "macos") {
                            // macOS: Try Chrome first, then Safari
                            std::process::Command::new(
                                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                            )
                            .arg(format!("--app={}", url))
                            .arg("--new-window")
                            .spawn()
                            .or_else(|_| {
                                // Fallback to open command with Safari
                                std::process::Command::new("open")
                                    .arg("-n") // New instance
                                    .arg("-a")
                                    .arg("Safari")
                                    .arg(&url)
                                    .spawn()
                            })
                        } else if cfg!(target_os = "windows") {
                            // Windows: Try Chrome, then Edge
                            std::process::Command::new("cmd")
                                .args(&["/C", "start", "chrome", &format!("--app={}", url)])
                                .spawn()
                                .or_else(|_| {
                                    std::process::Command::new("cmd")
                                        .args(&["/C", "start", "msedge", &format!("--app={}", url)])
                                        .spawn()
                                })
                        } else {
                            // Linux: Try chromium or google-chrome
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
                                // Fallback to normal browser open
                                if let Err(e) = open::that(&url) {
                                    tracing::warn!("Failed to open browser: {}", e);
                                } else {
                                    tracing::info!("Opening browser at {}", url);
                                }
                            }
                        }
                    } else {
                        // Normal browser open (existing behavior)
                        if let Err(e) = open::that(&url) {
                            tracing::warn!("Failed to open browser: {}", e);
                        } else {
                            tracing::info!("Opening browser at {}", url);
                        }
                    }
                });
            }
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
                // Keep the process alive for web server
                // Signal handler already set up above, just wait forever
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(KEEPALIVE_INTERVAL_SECS))
                        .await;
                }
            }
        }
    } else {
        tracing::info!("Running in standalone mode (web server only)");
        // Keep the process alive - the signal handler in the spawned task will handle shutdown
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
