use axum::{
    Router,
    routing::{get, post, delete},
};
use crate::web::server::AppState;

pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(super::handlers::get_status))
        .route("/processes", get(super::handlers::list_processes))
        .route("/processes", post(super::handlers::create_process))
        .route("/processes/:id", get(super::handlers::get_process))
        .route("/processes/:id", delete(super::handlers::remove_process))
        .route("/processes/:id/start", post(super::handlers::start_process))
        .route("/processes/:id/stop", post(super::handlers::stop_process))
        .route("/processes/:id/logs", get(super::handlers::get_process_logs))
        .route("/processes/:id/logs/stream", get(super::handlers::stream_logs))
}