use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::StreamExt;
use crate::process::{ProcessFilter, ProcessStateFilter, OutputStream};
use crate::messages::{CreateProcessRequest, StopProcessRequest};
use crate::web::server::AppState;

#[derive(Serialize)]
pub struct ServerStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    process_count: usize,
}

#[derive(Deserialize)]
pub struct ListProcessesQuery {
    state: Option<String>,
    name_pattern: Option<String>,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    stream: Option<String>,
    lines: Option<u32>,
}

pub async fn get_status(
    State(state): State<AppState>,
) -> Json<ServerStatus> {
    let processes = state.process_manager.list_processes(None).await;
    
    Json(ServerStatus {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        process_count: processes.len(),
    })
}

pub async fn list_processes(
    State(state): State<AppState>,
    Query(query): Query<ListProcessesQuery>,
) -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let filter = if query.state.is_some() || query.name_pattern.is_some() {
        Some(ProcessFilter {
            state: query.state.map(|s| match s.as_str() {
                "running" => ProcessStateFilter::Running,
                "stopped" => ProcessStateFilter::Stopped,
                "failed" => ProcessStateFilter::Failed,
                _ => ProcessStateFilter::All,
            }),
            name_pattern: query.name_pattern,
        })
    } else {
        None
    };
    
    let processes = state.process_manager.list_processes(filter).await;
    
    // Convert to JSON values
    let json_processes: Vec<serde_json::Value> = processes
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap_or(serde_json::json!({})))
        .collect();
    
    Ok(Json(json_processes))
}

pub async fn create_process(
    State(state): State<AppState>,
    Json(req): Json<CreateProcessRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let cwd = req.cwd.map(std::path::PathBuf::from);
    
    state.process_manager
        .create_process(req.id.clone(), req.command, req.args, req.env, cwd)
        .await
        .map(|_| {
            (StatusCode::CREATED, Json(serde_json::json!({
                "message": format!("Process '{}' created successfully", req.id)
            })))
        })
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.process_manager
        .get_process_status(id)
        .await
        .map(|status| Json(serde_json::to_value(status).unwrap()))
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn remove_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.process_manager
        .remove_process(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn start_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state.process_manager
        .start_process(id.clone())
        .await
        .map(|pid| {
            Json(serde_json::json!({
                "message": format!("Process '{}' started with PID {}", id, pid)
            }))
        })
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn stop_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<Option<StopProcessRequest>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let grace_period = req.and_then(|r| r.grace_period_ms);
    
    state.process_manager
        .stop_process(id, grace_period)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_process_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let stream = match query.stream.as_deref() {
        Some("stdout") => OutputStream::Stdout,
        Some("stderr") => OutputStream::Stderr,
        _ => OutputStream::Both,
    };
    
    state.process_manager
        .get_process_output(id, stream, query.lines)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn stream_logs(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // TODO: Implement real-time log streaming
    // For now, return a simple heartbeat stream
    let stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(std::time::Duration::from_secs(1))
    ).map(|_| {
        Ok(Event::default().data("heartbeat"))
    });
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}