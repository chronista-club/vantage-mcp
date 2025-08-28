use crate::messages::{CreateProcessRequest, StopProcessRequest};
use crate::process::{OutputStream, ProcessFilter, ProcessStateFilter};
use crate::web::server::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::StreamExt;

#[derive(Deserialize)]
pub struct ProcessConfigUpdate {
    pub auto_start: Option<bool>,
}

#[derive(Serialize)]
pub struct ServerStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    process_count: usize,
}

#[derive(Serialize)]
pub struct DashboardData {
    server: ServerInfo,
    stats: ProcessStats,
    processes: Vec<serde_json::Value>,
    recent_events: Vec<RecentEvent>,
    system_metrics: SystemMetrics,
}

#[derive(Serialize)]
pub struct ServerInfo {
    status: String,
    version: String,
    uptime_seconds: u64,
    current_time: u64,
}

#[derive(Serialize)]
pub struct ProcessStats {
    total: usize,
    running: usize,
    stopped: usize,
    failed: usize,
    auto_start_enabled: usize,
}

#[derive(Serialize)]
pub struct RecentEvent {
    timestamp: u64,
    process_id: String,
    event_type: String,
    message: String,
}

#[derive(Serialize)]
pub struct SystemMetrics {
    cpu_usage: f32,
    memory_usage: f32,
    process_manager_memory: usize,
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

pub async fn get_status(State(state): State<AppState>) -> Json<ServerStatus> {
    let processes = state.process_manager.list_processes(None).await;

    Json(ServerStatus {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        process_count: processes.len(),
    })
}

pub async fn get_dashboard(State(state): State<AppState>) -> Json<DashboardData> {
    let processes = state.process_manager.list_processes(None).await;

    // 統計を計算
    let mut stats = ProcessStats {
        total: processes.len(),
        running: 0,
        stopped: 0,
        failed: 0,
        auto_start_enabled: 0,
    };

    for process in &processes {
        match &process.state {
            crate::process::types::ProcessState::Running { .. } => stats.running += 1,
            crate::process::types::ProcessState::Stopped { .. } => stats.stopped += 1,
            crate::process::types::ProcessState::Failed { .. } => stats.failed += 1,
            crate::process::types::ProcessState::NotStarted => stats.stopped += 1,
        }

        // TODO: auto_startフラグは現在ProcessInfoに存在しないため、将来的に追加する必要がある
        // if process.auto_start {
        //     stats.auto_start_enabled += 1;
        // }
    }

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // ダミーのイベントデータ（将来的には実際のイベントを追跡）
    let recent_events = vec![RecentEvent {
        timestamp: current_time - 300,
        process_id: "example".to_string(),
        event_type: "started".to_string(),
        message: "プロセスが正常に開始されました".to_string(),
    }];

    // システムメトリクス（ダミーデータ、将来的には実際の値を取得）
    let system_metrics = SystemMetrics {
        cpu_usage: 12.5,
        memory_usage: 35.2,
        process_manager_memory: 1024 * 1024 * 50, // 50MB
    };

    // ProcessInfoをJSONに変換
    let processes_json: Vec<serde_json::Value> = processes
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect();

    Json(DashboardData {
        server: ServerInfo {
            status: "running".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0, // TODO: Track actual uptime
            current_time,
        },
        stats,
        processes: processes_json,
        recent_events,
        system_metrics,
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

    state
        .process_manager
        .create_process(req.id.clone(), req.command, req.args, req.env, cwd)
        .await
        .map(|_| {
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "message": format!("Process '{}' created successfully", req.id)
                })),
            )
        })
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state
        .process_manager
        .get_process_status(id)
        .await
        .map(|status| Json(serde_json::to_value(status).unwrap()))
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn remove_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .process_manager
        .remove_process(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn start_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    state
        .process_manager
        .start_process(id.clone())
        .await
        .map(|pid| {
            Json(serde_json::json!({
                "message": format!("Process '{}' started with PID {}", id, pid)
            }))
        })
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn update_process_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(config): Json<ProcessConfigUpdate>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .process_manager
        .update_process_config(id, config.auto_start)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn stop_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<Option<StopProcessRequest>>,
) -> Result<StatusCode, (StatusCode, String)> {
    let grace_period = req.and_then(|r| r.grace_period_ms);

    state
        .process_manager
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

    state
        .process_manager
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
    let stream = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        std::time::Duration::from_secs(1),
    ))
    .map(|_| Ok(Event::default().data("heartbeat")));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
