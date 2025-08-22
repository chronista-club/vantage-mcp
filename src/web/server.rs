use axum::{
    Router,
    response::Html,
    extract::State,
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::process::ProcessManager;
use tera::{Tera, Context};

pub async fn start_web_server(
    process_manager: ProcessManager,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(process_manager);
    
    // Try to bind to the specified port, or find an available one
    let (listener, actual_port) = bind_to_available_port(port).await?;
    
    let addr = SocketAddr::from(([127, 0, 0, 1], actual_port));
    tracing::info!("Web dashboard started on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn bind_to_available_port(preferred_port: u16) -> Result<(tokio::net::TcpListener, u16), Box<dyn std::error::Error>> {
    // First try the preferred port
    let addr = SocketAddr::from(([127, 0, 0, 1], preferred_port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Successfully bound to preferred port {}", preferred_port);
            Ok((listener, preferred_port))
        }
        Err(e) => {
            tracing::warn!("Port {} is already in use: {}. Trying to find an available port...", preferred_port, e);
            
            // Try a range of ports from preferred_port+1 to preferred_port+100
            for offset in 1..=100 {
                let try_port = preferred_port + offset;
                let addr = SocketAddr::from(([127, 0, 0, 1], try_port));
                
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => {
                        tracing::info!("Successfully bound to port {}", try_port);
                        return Ok((listener, try_port));
                    }
                    Err(_) => continue,
                }
            }
            
            // If still no port found, let the OS assign one
            let addr = SocketAddr::from(([127, 0, 0, 1], 0));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            let actual_port = listener.local_addr()?.port();
            tracing::info!("OS assigned port {}", actual_port);
            Ok((listener, actual_port))
        }
    }
}

fn create_app(process_manager: ProcessManager) -> Router {
    // Teraテンプレートエンジンを初期化
    let tera = match Tera::new("templates/**/*.tera") {
        Ok(t) => Arc::new(t),
        Err(e) => {
            tracing::error!("テンプレート初期化エラー: {}", e);
            panic!("テンプレートの初期化に失敗しました: {}", e);
        }
    };
    
    let app_state = AppState {
        process_manager: Arc::new(process_manager),
        tera,
    };
    
    Router::new()
        .route("/", axum::routing::get(index_handler))
        .nest("/api", super::api::create_api_routes())
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

#[derive(Clone)]
pub struct AppState {
    pub process_manager: Arc<ProcessManager>,
    pub tera: Arc<Tera>,
}

async fn index_handler(State(state): State<AppState>) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let context = Context::new();
    
    match state.tera.render("index.tera", &context) {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("テンプレートレンダリングエラー: {}", e);
            Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("テンプレートエラー: {}", e)))
        }
    }
}