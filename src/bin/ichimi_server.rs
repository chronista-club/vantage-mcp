use anyhow::Result;
use ichimi_server::IchimiServer;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut web_enabled = cfg!(feature = "web");  // Default to true if web feature is enabled
    let mut web_only = false;
    let mut web_port = 12700u16;
    let mut auto_open = true;  // Default to auto-open browser
    
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
                println!("  --web-only       Run only web dashboard (no MCP server)");
                println!("  --web-port PORT  Set web dashboard port (default: 12700)");
                println!("  --no-open        Don't automatically open browser for web dashboard");
                return Ok(());
            }
            "--version" | "-v" => {
                println!("ichimi-server v0.1.0-alpha1");
                return Ok(());
            }
            "--web" => {
                web_enabled = true;
            }
            "--no-web" => {
                web_enabled = false;
            }
            "--web-only" => {
                web_only = true;
                web_enabled = true;
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
    
    // Initialize tracing to stderr to avoid interfering with stdio protocol
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Ichimi Server");
    
    // Create a shared process manager
    let process_manager = ichimi_server::process::ProcessManager::new().await;
    
    // Start web server if enabled
    #[cfg(feature = "web")]
    if web_enabled {
        tracing::info!("Web dashboard enabled on port {}", web_port);
        
        // Open browser after a short delay to allow server to start
        if auto_open {
            let url = format!("http://localhost:{}", web_port);
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if let Err(e) = open::that(&url) {
                    tracing::warn!("Failed to open browser: {}", e);
                } else {
                    tracing::info!("Opening browser at {}", url);
                }
            });
        }
        
        if web_only {
            // Run only the web server
            if let Err(e) = ichimi_server::web::start_web_server(process_manager, web_port).await {
                tracing::error!("Web server error: {:?}", e);
                return Err(anyhow::anyhow!("Web server failed to start"));
            }
        } else {
            // Run both web and MCP servers
            let web_manager = process_manager.clone();
            let web_port_clone = web_port;
            
            // Spawn web server in background
            tokio::spawn(async move {
                if let Err(e) = ichimi_server::web::start_web_server(web_manager, web_port_clone).await {
                    tracing::error!("Web server error: {:?}", e);
                }
            });
            
            // Run MCP server with shared process manager
            let mut server = IchimiServer::new().await;
            server.set_process_manager(process_manager);
            
            let service = server
                .serve(stdio())
                .await
                .inspect_err(|e| {
                    tracing::error!("MCP Server error: {:?}", e);
                })?;
            
            service.waiting().await?;
        }
    } else {
        // Run MCP server only
        let mut server = IchimiServer::new().await;
        server.set_process_manager(process_manager);
        
        let service = server
            .serve(stdio())
            .await
            .inspect_err(|e| {
                tracing::error!("MCP Server error: {:?}", e);
            })?;
        
        service.waiting().await?;
    }
    
    #[cfg(not(feature = "web"))]
    if web_enabled {
        tracing::warn!("Web feature not enabled. Rebuild with --features web to enable dashboard.");
        
        // Run MCP server without web
        let mut server = IchimiServer::new().await;
        server.set_process_manager(process_manager);
        
        let service = server
            .serve(stdio())
            .await
            .inspect_err(|e| {
                tracing::error!("MCP Server error: {:?}", e);
            })?;
        
        service.waiting().await?;
    }
    
    Ok(())
}