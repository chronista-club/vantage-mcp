use crate::messages::clipboard::*;
use crate::messages::{CreateProcessRequest, StopProcessRequest, UpdateProcessRequest};
use crate::process::{OutputStream, ProcessFilter, ProcessStateFilter};
use crate::web::server::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use ichimi_persistence::{ClipboardItem, ProcessTemplate, TemplateVariable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::StreamExt;

#[derive(Deserialize)]
pub struct ProcessConfigUpdate {
    pub auto_start_on_restore: Option<bool>,
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

    // Create process with auto_start flags
    state
        .process_manager
        .create_process(
            req.id.clone(),
            req.command,
            req.args,
            req.env,
            cwd,
            req.auto_start_on_restore,
        )
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.clone()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": format!("Process '{}' created successfully", req.id)
        })),
    ))
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
        .update_process_config(id, config.auto_start_on_restore)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

/// Update process attributes
pub async fn update_process(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateProcessRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .process_manager
        .update_process(
            id,
            request.command,
            request.args,
            request.env,
            request.cwd,
            request.auto_start_on_restore,
        )
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

// Settings handlers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub color_mode: String,
    pub auto_refresh: bool,
    pub refresh_interval: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            color_mode: "dark".to_string(),
            auto_refresh: true,
            refresh_interval: 5000,
        }
    }
}

pub async fn get_settings(State(state): State<AppState>) -> Result<Json<Settings>, StatusCode> {
    // Persistence Managerから設定を取得
    let db_settings = state
        .process_manager
        .get_settings()
        .await
        .unwrap_or_default();

    // Convert from DB settings to handler settings
    let settings = Settings {
        color_mode: db_settings.theme,
        auto_refresh: db_settings.enable_auto_restart,
        refresh_interval: db_settings.auto_save_interval.unwrap_or(5000) as u32,
    };

    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<Settings>,
) -> Result<StatusCode, StatusCode> {
    // Convert to DB settings
    let db_settings = ichimi_persistence::Settings {
        theme: settings.color_mode,
        auto_save_interval: Some(settings.refresh_interval as u64),
        max_log_lines: None,
        enable_auto_restart: settings.auto_refresh,
        default_shell: None,
        env_variables: HashMap::new(),
        updated_at: chrono::Utc::now(),
    };

    // Persistence Managerに設定を保存
    state
        .process_manager
        .save_settings(db_settings)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// Template handlers

#[derive(Deserialize)]
pub struct ListTemplatesQuery {
    category: Option<String>,
    tags: Option<String>, // comma-separated tags
}

#[derive(Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub default_cwd: Option<String>,
    pub default_auto_start: bool,
    pub variables: Vec<TemplateVariable>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub default_cwd: Option<String>,
    pub default_auto_start: Option<bool>,
    pub variables: Option<Vec<TemplateVariable>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct InstantiateTemplateRequest {
    pub process_id: String,
    pub values: HashMap<String, String>,
}

pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<Vec<ProcessTemplate>>, StatusCode> {
    // タグをカンマ区切りで分割
    let tags = query
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // カテゴリとタグで検索
    state
        .process_manager
        .search_templates(query.category, tags)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list templates: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProcessTemplate>, StatusCode> {
    state
        .process_manager
        .get_template(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let template = ProcessTemplate {
        id: None,
        template_id: req.id.clone(),
        name: req.name,
        description: req.description,
        category: req.category,
        command: req.command,
        args: req.args,
        env: req.env,
        default_cwd: req.default_cwd,
        default_auto_start: req.default_auto_start,
        variables: req.variables,
        tags: req.tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .process_manager
        .save_template(template)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": format!("Template '{}' created successfully", req.id)
        })),
    ))
}

pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 既存のテンプレートを取得
    let mut template = state
        .process_manager
        .get_template(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    // 更新する
    if let Some(name) = req.name {
        template.name = name;
    }
    if let Some(description) = req.description {
        template.description = Some(description);
    }
    if let Some(category) = req.category {
        template.category = Some(category);
    }
    if let Some(command) = req.command {
        template.command = command;
    }
    if let Some(args) = req.args {
        template.args = args;
    }
    if let Some(env) = req.env {
        template.env = env;
    }
    if let Some(default_cwd) = req.default_cwd {
        template.default_cwd = Some(default_cwd);
    }
    if let Some(default_auto_start) = req.default_auto_start {
        template.default_auto_start = default_auto_start;
    }
    if let Some(variables) = req.variables {
        template.variables = variables;
    }
    if let Some(tags) = req.tags {
        template.tags = tags;
    }

    template.updated_at = chrono::Utc::now();

    state
        .process_manager
        .save_template(template)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(StatusCode::OK)
}

pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .process_manager
        .delete_template(&id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn instantiate_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<InstantiateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    // テンプレートを取得
    let template = state
        .process_manager
        .get_template(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    // テンプレートからプロセスを生成
    let process_info = template
        .instantiate(req.process_id.clone(), req.values)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // プロセスを作成
    state
        .process_manager
        .create_process(
            process_info.process_id.clone(),
            process_info.command,
            process_info.args,
            process_info.env,
            process_info.cwd.map(PathBuf::from),
            process_info.auto_start_on_restore,
        )
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": format!("Process '{}' created from template '{}'", req.process_id, id)
        })),
    ))
}

// ========================================
// Clipboard Handlers
// ========================================

/// Get the latest clipboard item
pub async fn get_clipboard(
    State(state): State<AppState>,
) -> Result<Json<ClipboardResponse>, (StatusCode, String)> {
    let item_opt = state
        .persistence_manager
        .get_latest_clipboard_item()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let item = item_opt.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            "No clipboard items found".to_string(),
        )
    })?;

    Ok(Json(ClipboardResponse {
        id: item.clipboard_id,
        content: item.content,
        filename: item.filename,
        created_at: item.created_at.to_rfc3339(),
        updated_at: item.updated_at.to_rfc3339(),
        content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
        tags: item.tags,
    }))
}

/// Get clipboard history
#[derive(Deserialize)]
pub struct ClipboardHistoryQuery {
    limit: Option<usize>,
}

pub async fn get_clipboard_history(
    Query(query): Query<ClipboardHistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<ClipboardHistoryResponse>, (StatusCode, String)> {
    let items = state
        .persistence_manager
        .get_clipboard_history(Some(query.limit.unwrap_or(100)))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response_items: Vec<ClipboardResponse> = items
        .into_iter()
        .map(|item| ClipboardResponse {
            id: item.clipboard_id,
            content: item.content,
            filename: item.filename,
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
            content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
            tags: item.tags,
        })
        .collect();

    Ok(Json(ClipboardHistoryResponse {
        total_count: response_items.len(),
        items: response_items,
    }))
}

/// Set clipboard text content
pub async fn set_clipboard_text(
    State(state): State<AppState>,
    Json(req): Json<SetClipboardTextRequest>,
) -> Result<Json<ClipboardResponse>, (StatusCode, String)> {
    let item = state
        .persistence_manager
        .set_clipboard_text(req.content)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Update tags if provided
    let final_item = if !req.tags.is_empty() {
        let mut updated_item = item;
        updated_item.tags = req.tags;
        // Save updated item
        state
            .persistence_manager
            .save_clipboard_item(&updated_item)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        updated_item
    } else {
        item
    };

    Ok(Json(ClipboardResponse {
        id: final_item.clipboard_id,
        content: final_item.content,
        filename: final_item.filename,
        created_at: final_item.created_at.to_rfc3339(),
        updated_at: final_item.updated_at.to_rfc3339(),
        content_type: final_item
            .content_type
            .unwrap_or_else(|| "text".to_string()),
        tags: final_item.tags,
    }))
}

/// Set clipboard file content
pub async fn set_clipboard_file(
    State(state): State<AppState>,
    Json(req): Json<SetClipboardFileRequest>,
) -> Result<Json<ClipboardResponse>, (StatusCode, String)> {
    let item = ClipboardItem {
        id: None,
        clipboard_id: ichimi_persistence::generate_id(),
        content: req.content,
        filename: Some(req.filename),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        content_type: Some("file".to_string()),
        tags: req.tags,
    };

    state
        .persistence_manager
        .save_clipboard_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ClipboardResponse {
        id: item.clipboard_id,
        content: item.content,
        filename: item.filename,
        created_at: item.created_at.to_rfc3339(),
        updated_at: item.updated_at.to_rfc3339(),
        content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
        tags: item.tags,
    }))
}

/// Search clipboard items
pub async fn search_clipboard(
    Query(req): Query<SearchClipboardRequest>,
    State(state): State<AppState>,
) -> Result<Json<ClipboardHistoryResponse>, (StatusCode, String)> {
    // Simple search implementation using get_clipboard_history
    let all_items = state
        .persistence_manager
        .get_clipboard_history(Some(req.limit.unwrap_or(50)))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Filter items based on query
    let items: Vec<_> = all_items
        .into_iter()
        .filter(|item| {
            item.content.contains(&req.query)
                || item
                    .filename
                    .as_ref()
                    .is_some_and(|f| f.contains(&req.query))
                || item.tags.iter().any(|tag| tag.contains(&req.query))
        })
        .collect();

    let response_items: Vec<ClipboardResponse> = items
        .into_iter()
        .map(|item| ClipboardResponse {
            id: item.clipboard_id,
            content: item.content,
            filename: item.filename,
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
            content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
            tags: item.tags,
        })
        .collect();

    Ok(Json(ClipboardHistoryResponse {
        total_count: response_items.len(),
        items: response_items,
    }))
}

/// Clear all clipboard items
pub async fn clear_clipboard(
    State(state): State<AppState>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    state
        .persistence_manager
        .clear_clipboard()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        StatusCode::NO_CONTENT,
        "All clipboard items cleared".to_string(),
    ))
}
