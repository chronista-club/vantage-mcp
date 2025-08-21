use axum::Router;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::process::ProcessManager;

pub async fn start_web_server(
    process_manager: ProcessManager,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(process_manager);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Web dashboard starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn create_app(process_manager: ProcessManager) -> Router {
    Router::new()
        .nest("/api", super::api::create_api_routes())
        .nest_service("/", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(process_manager))
}