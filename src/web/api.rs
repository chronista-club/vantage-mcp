use crate::web::server::AppState;
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(super::handlers::get_status))
        .route("/dashboard", get(super::handlers::get_dashboard))
        .route("/processes", get(super::handlers::list_processes))
        .route("/processes", post(super::handlers::create_process))
        .route("/processes/:id", get(super::handlers::get_process))
        .route("/processes/:id", delete(super::handlers::remove_process))
        .route("/processes/:id", put(super::handlers::update_process))
        .route("/processes/:id/start", post(super::handlers::start_process))
        .route("/processes/:id/stop", post(super::handlers::stop_process))
        .route(
            "/processes/:id/config",
            patch(super::handlers::update_process_config),
        )
        .route(
            "/processes/:id/logs",
            get(super::handlers::get_process_logs),
        )
        .route(
            "/processes/:id/logs/stream",
            get(super::handlers::stream_logs),
        )
}
