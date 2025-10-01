use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod ci;
pub mod error;
pub mod events;
pub mod learning;
pub mod messages;
pub mod process;
pub mod security;
#[cfg(feature = "web")]
pub mod web;

pub use error::{IchimiError, IchimiResult};

use ci::CiMonitor;
use events::EventSystem;
use learning::LearningEngine;
use messages::*;
use process::ProcessManager;

#[derive(Clone)]
pub struct IchimiServer {
    start_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    process_manager: ProcessManager,
    #[allow(dead_code)]
    event_system: Arc<EventSystem>,
    learning_engine: Arc<LearningEngine>,
    #[allow(dead_code)]
    ci_monitor: Arc<CiMonitor>,
    tool_router: ToolRouter<IchimiServer>,
}

#[tool_router]
impl IchimiServer {
    pub async fn new() -> anyhow::Result<Self> {
        tracing::info!("Initializing IchimiServer");

        // ProcessManagerを初期化
        tracing::debug!("Initializing process manager");
        let process_manager = ProcessManager::new().await;

        // イベントシステムを初期化（Database依存を削除）
        tracing::debug!("Initializing event system");
        let event_system = Arc::new(EventSystem::new());

        // 学習エンジンを初期化（Database依存を削除）
        tracing::debug!("Initializing learning engine");
        let learning_engine = Arc::new(LearningEngine::new(event_system.clone()));

        // CI監視を初期化
        tracing::debug!("Initializing CI monitor");
        let ci_monitor = Arc::new(CiMonitor::new(None, Some(30)));

        tracing::info!("IchimiServer initialization complete");
        Ok(Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager,
            event_system,
            learning_engine,
            ci_monitor,
            tool_router: Self::tool_router(),
        })
    }

    pub fn set_process_manager(&mut self, manager: ProcessManager) {
        self.process_manager = manager;
    }

    /// Create IchimiServer with existing ProcessManager (shares database)
    pub async fn with_process_manager(process_manager: ProcessManager) -> anyhow::Result<Self> {
        tracing::info!("Initializing IchimiServer with existing ProcessManager");

        // Initialize event system
        let event_system = Arc::new(EventSystem::new());

        // Initialize learning engine
        let learning_engine = Arc::new(LearningEngine::new(event_system.clone()));

        // Start learning
        if let Err(e) = learning_engine.start_learning().await {
            tracing::warn!("Failed to start learning engine: {}", e);
            // 学習エンジンの失敗は致命的ではないので、警告のみ
        } else {
            tracing::info!("Learning engine started successfully");
        }

        tracing::info!("IchimiServer initialization complete");

        // CI監視を初期化（2回目の初期化）
        let ci_monitor_2 = Arc::new(CiMonitor::new(None, Some(30)));

        Ok(Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager,
            event_system,
            learning_engine,
            ci_monitor: ci_monitor_2,
            tool_router: Self::tool_router(),
        })
    }

    /// サーバー終了時の処理
    pub async fn shutdown(&self) -> std::result::Result<(), String> {
        tracing::info!("Shutting down IchimiServer");

        // シャットダウン時にプロセス状態を保存（YAMLスナップショット）
        self.process_manager
            .create_yaml_snapshot_on_shutdown()
            .await
            .map_err(|e| format!("Failed to save process snapshot on shutdown: {e}"))?;

        tracing::info!("Shutdown complete");
        Ok(())
    }

    #[tool(description = "Echo the input message back")]
    fn echo(
        &self,
        Parameters(EchoRequest { message }): Parameters<EchoRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Echo: {message}"
        ))]))
    }

    #[tool(description = "Simple ping/pong health check")]
    fn ping(&self) -> std::result::Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("pong")]))
    }

    #[tool(description = "Get the current server status")]
    async fn get_status(&self) -> std::result::Result<CallToolResult, McpError> {
        let start_time = self.start_time.lock().await;
        let uptime = chrono::Utc::now() - *start_time;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Status: running\nVersion: 0.1.0\nUptime: {} seconds\nTools: echo, ping, get_status, create_process, start_process, stop_process, get_process_status, get_process_output, list_processes, remove_process",
            uptime.num_seconds()
        ))]))
    }

    #[tool(description = "Create and register a new process")]
    async fn create_process(
        &self,
        Parameters(CreateProcessRequest {
            id,
            command,
            args,
            env,
            cwd,
            auto_start_on_restore,
        }): Parameters<CreateProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let cwd_path = cwd.map(std::path::PathBuf::from);

        // Create the process
        self.process_manager
            .create_process(
                id.clone(),
                command,
                args,
                env,
                cwd_path,
                auto_start_on_restore,
            )
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{id}' created successfully"
        ))]))
    }

    #[tool(description = "Start a registered process")]
    async fn start_process(
        &self,
        Parameters(StartProcessRequest { id }): Parameters<StartProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let pid = self
            .process_manager
            .start_process(id.clone())
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{id}' started with PID {pid}"
        ))]))
    }

    #[tool(description = "Stop a running process")]
    async fn stop_process(
        &self,
        Parameters(McpStopProcessRequest {
            id,
            grace_period_ms,
        }): Parameters<McpStopProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .stop_process(id.clone(), grace_period_ms)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{id}' stopped successfully"
        ))]))
    }

    #[tool(description = "Get process status and metrics")]
    async fn get_process_status(
        &self,
        Parameters(GetProcessStatusRequest { id }): Parameters<GetProcessStatusRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let status = self
            .process_manager
            .get_process_status(id)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let json = serde_json::to_string_pretty(&status).map_err(|e| McpError {
            message: format!("Failed to serialize status: {e}").into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get process output (stdout/stderr)")]
    async fn get_process_output(
        &self,
        Parameters(GetProcessOutputRequest { id, stream, lines }): Parameters<
            GetProcessOutputRequest,
        >,
    ) -> std::result::Result<CallToolResult, McpError> {
        let output = self
            .process_manager
            .get_process_output(id, stream, lines)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(
            output.join("\n"),
        )]))
    }

    #[tool(description = "List all managed processes")]
    async fn list_processes(
        &self,
        Parameters(ListProcessesRequest { filter }): Parameters<ListProcessesRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let processes = self.process_manager.list_processes(filter).await;

        let json = serde_json::to_string_pretty(&processes).map_err(|e| McpError {
            message: format!("Failed to serialize processes: {e}").into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Remove a process from management")]
    async fn remove_process(
        &self,
        Parameters(RemoveProcessRequest { id }): Parameters<RemoveProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .remove_process(id.clone())
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{id}' removed successfully"
        ))]))
    }

    #[tool(description = "Export all processes to a JSON file for backup/persistence")]
    async fn export_processes(
        &self,
        Parameters(ExportProcessesRequest { file_path }): Parameters<ExportProcessesRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let path = self
            .process_manager
            .export_processes(file_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Processes exported successfully to {path}"
        ))]))
    }

    #[tool(description = "Import processes from a JSON file")]
    async fn import_processes(
        &self,
        Parameters(ImportProcessesRequest { file_path }): Parameters<ImportProcessesRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .import_processes(&file_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Processes imported successfully from {file_path}"
        ))]))
    }

    #[tool(
        description = "Create a snapshot of the entire database (processes, templates, clipboard)"
    )]
    async fn create_snapshot(&self) -> std::result::Result<CallToolResult, McpError> {
        let path = self
            .process_manager
            .create_snapshot()
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Snapshot created successfully at {path}"
        ))]))
    }

    #[tool(description = "Restore the database from the latest snapshot")]
    async fn restore_snapshot(&self) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .restore_snapshot()
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(
            "Snapshot restored successfully".to_string(),
        )]))
    }

    #[tool(description = "Export processes to YAML format")]
    async fn export_yaml(
        &self,
        Parameters(ExportYamlRequest {
            file_path,
            only_auto_start,
        }): Parameters<ExportYamlRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let path = self
            .process_manager
            .export_yaml(file_path, only_auto_start)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let message = if only_auto_start {
            format!("Auto-start processes exported to YAML at {path}")
        } else {
            format!("All processes exported to YAML at {path}")
        };

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(description = "Import processes from YAML format")]
    async fn import_yaml(
        &self,
        Parameters(ImportYamlRequest { file_path }): Parameters<ImportYamlRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .import_yaml(&file_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Processes imported successfully from YAML file {file_path}"
        ))]))
    }

    #[tool(description = "Create a snapshot in specified format (yaml or surql)")]
    async fn create_formatted_snapshot(
        &self,
        Parameters(CreateSnapshotRequest { file_path, format }): Parameters<CreateSnapshotRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let path = match format {
            SnapshotFormat::Yaml => {
                self.process_manager
                    .export_yaml(file_path, true) // Only auto-start for snapshots
                    .await
            }
            SnapshotFormat::Surql => self.process_manager.export_processes(file_path).await,
        }
        .map_err(|e| McpError {
            message: e.into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Snapshot created successfully at {path} (format: {format:?})"
        ))]))
    }

    #[tool(description = "Update process configuration (auto_start flags)")]
    async fn update_process_config(
        &self,
        Parameters(UpdateProcessConfigRequest {
            id,
            auto_start_on_restore,
        }): Parameters<UpdateProcessConfigRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .update_process_config(id.clone(), auto_start_on_restore)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let mut message = format!("Process '{id}' configuration updated");
        if let Some(value) = auto_start_on_restore {
            message.push_str(&format!(" - auto_start_on_restore set to {value}"));
        }

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(
        description = "Update process attributes (command, args, env, cwd, and auto_start flags)"
    )]
    async fn update_process(
        &self,
        Parameters(UpdateProcessRequest {
            id,
            command,
            args,
            env,
            cwd,
            auto_start_on_restore,
        }): Parameters<UpdateProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .update_process(
                id.clone(),
                command.clone(),
                args.clone(),
                env.clone(),
                cwd.clone(),
                auto_start_on_restore,
            )
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let mut updates = Vec::new();
        if command.is_some() {
            updates.push("command");
        }
        if args.is_some() {
            updates.push("args");
        }
        if env.is_some() {
            updates.push("env");
        }
        if cwd.is_some() {
            updates.push("cwd");
        }
        if auto_start_on_restore.is_some() {
            updates.push("auto_start_on_restore");
        }

        let message = if updates.is_empty() {
            format!("Process '{id}' - no attributes updated")
        } else {
            format!("Process '{}' updated: {}", id, updates.join(", "))
        };

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(description = "Get smart suggestions for next actions based on learning")]
    async fn get_suggestions(
        &self,
        Parameters(GetSuggestionsRequest { current_process }): Parameters<GetSuggestionsRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let suggestions = self
            .learning_engine
            .get_suggestions(current_process.as_deref())
            .await
            .map_err(|e| McpError {
                message: format!("{e}").into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        if suggestions.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No suggestions available at this time.",
            )]));
        }

        let mut result = String::from("Smart Suggestions:\n\n");
        for (i, suggestion) in suggestions.iter().enumerate() {
            result.push_str(&format!(
                "{}. {}\n   Action: {:?}\n   Confidence: {:.0}%\n   Reason: {}\n\n",
                i + 1,
                suggestion.message,
                suggestion.action,
                suggestion.confidence * 100.0,
                suggestion.reason
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    // CI監視ツール

    #[tool(description = "List recent CI/CD runs from GitHub Actions")]
    async fn list_ci_runs(
        &self,
        Parameters(request): Parameters<ListCiRunsRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Listing CI runs (limit: {})", request.limit);

        let ci_monitor = if let Some(repo) = request.repo {
            CiMonitor::new(Some(repo), None)
        } else {
            CiMonitor::new(None, None)
        };

        match ci_monitor.get_latest_runs(request.limit).await {
            Ok(runs) => {
                let response = ListCiRunsResponse {
                    total_count: runs.len(),
                    runs: runs
                        .into_iter()
                        .map(|run| CiRunResponse {
                            id: run.id,
                            name: run.name,
                            workflow_name: run.workflow_name,
                            branch: run.branch,
                            event: run.event,
                            status: format!("{:?}", run.status),
                            conclusion: run.conclusion.map(|c| format!("{c:?}")),
                            created_at: run.created_at,
                            updated_at: run.updated_at,
                            duration: run.duration,
                            url: run.url,
                        })
                        .collect(),
                };

                let json = serde_json::to_string_pretty(&response)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => {
                tracing::error!("Failed to list CI runs: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to list CI runs: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Get detailed information about a specific CI run")]
    async fn get_ci_run_details(
        &self,
        Parameters(request): Parameters<GetCiRunDetailsRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Getting details for CI run {}", request.run_id);

        let ci_monitor = if let Some(repo) = request.repo {
            CiMonitor::new(Some(repo), None)
        } else {
            CiMonitor::new(None, None)
        };

        match ci_monitor.get_run_details(request.run_id).await {
            Ok(details) => Ok(CallToolResult::success(vec![Content::text(details)])),
            Err(e) => {
                tracing::error!("Failed to get CI run details: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to get CI run details: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Get logs from failed jobs in a CI run")]
    async fn get_ci_failed_logs(
        &self,
        Parameters(request): Parameters<GetCiFailedLogsRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Getting failed logs for CI run {}", request.run_id);

        let ci_monitor = if let Some(repo) = request.repo {
            CiMonitor::new(Some(repo), None)
        } else {
            CiMonitor::new(None, None)
        };

        match ci_monitor.get_failed_logs(request.run_id).await {
            Ok(logs) => Ok(CallToolResult::success(vec![Content::text(logs)])),
            Err(e) => {
                tracing::error!("Failed to get CI failed logs: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to get CI failed logs: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Wait for a CI run to complete and return its final status")]
    async fn wait_for_ci_completion(
        &self,
        Parameters(request): Parameters<WaitForCiCompletionRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!(
            "Waiting for CI run {} to complete (timeout: {}s)",
            request.run_id,
            request.timeout_secs
        );

        let ci_monitor = if let Some(repo) = request.repo {
            CiMonitor::new(Some(repo), None)
        } else {
            CiMonitor::new(None, None)
        };

        match ci_monitor
            .wait_for_completion(request.run_id, Some(request.timeout_secs))
            .await
        {
            Ok(run) => {
                let response = CiRunResponse {
                    id: run.id,
                    name: run.name,
                    workflow_name: run.workflow_name,
                    branch: run.branch,
                    event: run.event,
                    status: format!("{:?}", run.status),
                    conclusion: run.conclusion.map(|c| format!("{c:?}")),
                    created_at: run.created_at,
                    updated_at: run.updated_at,
                    duration: run.duration,
                    url: run.url,
                };

                let json = serde_json::to_string_pretty(&response)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => {
                tracing::error!("Failed to wait for CI completion: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to wait for CI completion: {e}"),
                    None,
                ))
            }
        }
    }

    #[tool(description = "Start monitoring CI runs in the background")]
    async fn start_ci_monitoring(
        &self,
        Parameters(request): Parameters<StartCiMonitoringRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!(
            "Starting CI monitoring with {}s interval",
            request.poll_interval
        );

        let ci_monitor = if let Some(repo) = request.repo {
            CiMonitor::new(Some(repo), Some(request.poll_interval))
        } else {
            CiMonitor::new(None, Some(request.poll_interval))
        };

        ci_monitor.start_monitoring().await;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "CI monitoring started with {}s polling interval",
            request.poll_interval
        ))]))
    }

    // クリップボード関連ツール

    #[tool(description = "Set clipboard content with text")]
    async fn set_clipboard_text(
        &self,
        Parameters(SetClipboardTextRequest { content, tags }): Parameters<SetClipboardTextRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let persistence = self.process_manager.persistence_manager();

        let mut item = persistence
            .set_clipboard_text(content)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        if !tags.is_empty() {
            item.tags = tags;
            persistence
                .save_clipboard_item(&item)
                .await
                .map_err(|e| McpError {
                    message: e.into(),
                    code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                    data: None,
                })?;
        }

        let response = ClipboardResponse {
            id: item.clipboard_id,
            content: item.content,
            filename: item.filename,
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
            content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
            tags: item.tags,
        };

        let json = serde_json::to_string_pretty(&response).map_err(|e| McpError {
            message: format!("Failed to serialize response: {e}").into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get the latest clipboard content")]
    async fn get_clipboard(
        &self,
        Parameters(_request): Parameters<GetClipboardRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let persistence = self.process_manager.persistence_manager();

        let item = persistence
            .get_latest_clipboard_item()
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?
            .ok_or_else(|| McpError {
                message: "No clipboard item found".into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let response = ClipboardResponse {
            id: item.clipboard_id,
            content: item.content,
            filename: item.filename,
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
            content_type: item.content_type.unwrap_or_else(|| "text".to_string()),
            tags: item.tags,
        };

        let json = serde_json::to_string_pretty(&response).map_err(|e| McpError {
            message: format!("Failed to serialize response: {e}").into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Open the Ichimi web console in your browser")]
    async fn open_web_console(
        &self,
        Parameters(request): Parameters<messages::OpenWebConsoleRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let port = request.port.unwrap_or(12700);
        let auto_open = request.auto_open.unwrap_or(true);

        tracing::info!("Opening web console on port {}", port);

        // Check if web server is already running by trying to connect
        let url = format!("http://localhost:{port}");

        // Try to check if the server is already running
        match reqwest::get(&format!("{url}/api/status")).await {
            Ok(response) if response.status().is_success() => {
                // Server is already running
                if auto_open {
                    if let Err(e) = open::that(&url) {
                        tracing::warn!("Failed to open browser: {}", e);
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Web console is already running at {url}. Please open it manually."
                        ))]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Web console is already running at {url}"
                ))]))
            }
            _ => {
                // Server is not running, we need to inform the user
                // Note: In MCP context, we cannot spawn a long-running web server
                // We should guide the user to run it separately
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Web console is not running. Please start Ichimi with web mode:\n\
                     \n\
                     ichimi --web-only --web-port {port}\n\
                     \n\
                     Or use the default port:\n\
                     ichimi --web-only\n\
                     \n\
                     The web console will be available at {url}"
                ))]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for IchimiServer {
    fn get_info(&self) -> ServerInfo {
        tracing::info!("MCP client requesting server info");
        let info = ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "ichimi-server".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some(
                "Ichimi Server - A powerful process management server for Claude Code via MCP."
                    .to_string(),
            ),
        };
        tracing::debug!("Returning server info: {:?}", info.server_info);
        info
    }
}
