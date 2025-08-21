use anyhow::Result;
use ichimi_server::IchimiServer;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut web_enabled = false;
    let mut web_only = false;
    let mut web_port = 12700u16;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--web" => {
                web_enabled = true;
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
    let process_manager = ichimi_server::process::ProcessManager::new();
    
    // Start web server if enabled
    #[cfg(feature = "web")]
    if web_enabled {
        tracing::info!("Web dashboard enabled on port {}", web_port);
        
        if web_only {
            // Run only the web server
            if let Err(e) = ichimi_server::web::start_web_server(process_manager, web_port).await {
                tracing::error!("Web server error: {:?}", e);
                return Err(anyhow::anyhow!("Web server failed to start"));
            }
        } else {
            // Run both web and MCP servers
            let web_manager = process_manager.clone();
            
            // Spawn web server in background
            tokio::spawn(async move {
                if let Err(e) = ichimi_server::web::start_web_server(web_manager, web_port).await {
                    tracing::error!("Web server error: {:?}", e);
                }
            });
            
            // Run MCP server with shared process manager
            let mut server = IchimiServer::new();
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
        let mut server = IchimiServer::new();
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
        let mut server = IchimiServer::new();
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