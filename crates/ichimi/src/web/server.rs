use crate::process::ProcessManager;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use ichimi_persistence::PersistenceManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub async fn start_web_server(
    process_manager: ProcessManager,
    persistence_manager: Arc<PersistenceManager>,
    port: u16,
) -> Result<u16, Box<dyn std::error::Error>> {
    let app = create_app(process_manager, persistence_manager);

    // Try to bind to the specified port, or find an available one
    let (listener, actual_port) = bind_to_available_port(port).await?;

    let addr = SocketAddr::from(([127, 0, 0, 1], actual_port));
    tracing::info!("Web dashboard started on http://{}", addr);

    // Spawn the server in a background task
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Web server error: {}", e);
        }
    });

    Ok(actual_port)
}

async fn bind_to_available_port(
    preferred_port: u16,
) -> Result<(tokio::net::TcpListener, u16), Box<dyn std::error::Error>> {
    // First try the preferred port
    let addr = SocketAddr::from(([127, 0, 0, 1], preferred_port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Successfully bound to preferred port {}", preferred_port);
            Ok((listener, preferred_port))
        }
        Err(e) => {
            tracing::warn!(
                "Port {} is already in use: {}. Trying to find an available port...",
                preferred_port,
                e
            );

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

fn create_app(
    process_manager: ProcessManager,
    persistence_manager: Arc<PersistenceManager>,
) -> Router {
    let app_state = AppState {
        process_manager: Arc::new(process_manager),
        persistence_manager,
    };

    Router::new()
        .route("/", axum::routing::get(index_handler))
        .nest("/api", super::api::create_api_routes())
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

#[derive(Clone)]
pub struct AppState {
    pub process_manager: Arc<ProcessManager>,
    pub persistence_manager: Arc<PersistenceManager>,
}

async fn index_handler() -> impl IntoResponse {
    // Serve the built web app from embedded asset
    match super::assets::Asset::get("ui/web/dist/index.html") {
        Some(content) => Html(
            std::str::from_utf8(content.data.as_ref())
                .unwrap_or("Error loading page")
                .to_string(),
        ),
        None => Html("Error: index.html not found".to_string()),
    }
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    use super::assets::Asset;

    // URIからパスを取得
    let path = uri.path();

    // パスの正規化
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else if path.starts_with('/') {
        &path[1..]
    } else {
        path
    };

    tracing::debug!("Static file request: {} -> {}", uri.path(), path);

    // Webビルドファイルをチェック（ui/web/dist/ディレクトリ）
    let dist_path = format!("ui/web/dist/{}", path);
    if let Some((data, mime)) = Asset::get_with_mime(&dist_path) {
        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", mime)
            .body(axum::body::Body::from(data))
            .unwrap();
    }

    // ファイルが見つからない場合は404を返す
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(axum::body::Body::from("404 Not Found"))
        .unwrap()
}
