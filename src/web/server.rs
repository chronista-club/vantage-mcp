use crate::process::ProcessManager;
use axum::{Router, extract::State, response::Html};
use std::net::SocketAddr;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::{cors::CorsLayer, services::ServeDir};

// テンプレートファイルをバイナリに埋め込む

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

fn create_app(process_manager: ProcessManager) -> Router {
    // Teraテンプレートエンジンを初期化（埋め込みテンプレートを使用）
    let mut tera = Tera::default();

    // 実行時にテンプレートファイルを読み込む
    let base_template = include_str!("../../templates/base.tera");
    let index_template = include_str!("../../templates/index.tera");

    // 埋め込んだテンプレートを追加
    if let Err(e) = tera.add_raw_template("base.tera", base_template) {
        tracing::error!("baseテンプレートの追加エラー: {}", e);
        panic!("baseテンプレートの追加に失敗しました: {}", e);
    }

    if let Err(e) = tera.add_raw_template("index.tera", index_template) {
        tracing::error!("indexテンプレートの追加エラー: {}", e);
        panic!("indexテンプレートの追加に失敗しました: {}", e);
    }

    let tera = Arc::new(tera);

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

async fn index_handler(
    State(state): State<AppState>,
) -> Result<Html<String>, (axum::http::StatusCode, String)> {
    let context = Context::new();

    match state.tera.render("index.tera", &context) {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            tracing::error!("テンプレートレンダリングエラー: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("テンプレートエラー: {}", e),
            ))
        }
    }
}
