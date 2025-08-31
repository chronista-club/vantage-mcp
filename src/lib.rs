use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod ci;
pub mod db;
pub mod error;
pub mod events;
pub mod learning;
pub mod messages;
pub mod persistence;
pub mod process;
pub mod security;
#[cfg(feature = "web")]
pub mod web;

pub use error::{IchimiError, IchimiResult};

use ci::CiMonitor;
use db::Database;
use events::EventSystem;
use learning::LearningEngine;
use messages::*;
use process::ProcessManager;

#[derive(Clone)]
pub struct IchimiServer {
    start_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    process_manager: ProcessManager,
    database: Arc<Database>,
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

        // データベースを初期化
        tracing::debug!("Initializing database");
        let database = Arc::new(Database::new().await.map_err(|e| {
            tracing::error!("Failed to initialize database: {}", e);
            anyhow::anyhow!("Database initialization failed: {}", e)
        })?);
        tracing::debug!("Database initialized successfully");

        // 起動時に既存データを復元
        database.restore_on_startup().await.unwrap_or_else(|e| {
            tracing::warn!("Failed to restore data on startup: {}", e);
        });

        // イベントシステムを初期化
        tracing::debug!("Initializing event system");
        let event_system = Arc::new(EventSystem::new(database.clone()));

        // 学習エンジンを初期化
        tracing::debug!("Initializing learning engine");
        let learning_engine = Arc::new(LearningEngine::new(database.clone(), event_system.clone()));

        // 学習を開始
        tracing::debug!("Starting learning engine");
        if let Err(e) = learning_engine.start_learning().await {
            tracing::warn!("Failed to start learning engine: {}", e);
            // 学習エンジンの失敗は致命的ではないので、警告のみ
        } else {
            tracing::info!("Learning engine started successfully");
        }

        // ProcessManagerを共有Databaseインスタンスで初期化
        tracing::debug!("Initializing process manager with shared database");
        let process_manager = ProcessManager::with_database(database.clone()).await;

        // CI監視を初期化
        tracing::debug!("Initializing CI monitor");
        let ci_monitor = Arc::new(CiMonitor::new(None, Some(30)));

        tracing::info!("IchimiServer initialization complete");
        Ok(Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager,
            database,
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

        // Get database from ProcessManager
        let database = process_manager.database();

        // Initialize event system with shared database
        let event_system = Arc::new(EventSystem::new(database.clone()));

        // Initialize learning engine with shared database
        let learning_engine = Arc::new(LearningEngine::new(database.clone(), event_system.clone()));

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
            database,
            event_system,
            learning_engine,
            ci_monitor: ci_monitor_2,
            tool_router: Self::tool_router(),
        })
    }

    /// サーバー終了時の処理
    pub async fn shutdown(&self) -> std::result::Result<(), String> {
        tracing::info!("Shutting down IchimiServer");

        // データベースをバックアップ
        self.database
            .backup_on_shutdown()
            .await
            .map_err(|e| format!("Failed to backup data on shutdown: {e}"))?;

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
            auto_start_on_create,
            auto_start_on_restore,
        }): Parameters<CreateProcessRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let cwd_path = cwd.map(std::path::PathBuf::from);

        // Create the process with auto_start flags
        self.process_manager
            .create_process(
                id.clone(),
                command,
                args,
                env,
                cwd_path,
                auto_start_on_create,
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
        Parameters(StopProcessRequest {
            id,
            grace_period_ms,
        }): Parameters<StopProcessRequest>,
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

    #[tool(description = "Update process configuration (auto_start flags)")]
    async fn update_process_config(
        &self,
        Parameters(UpdateProcessConfigRequest {
            id,
            auto_start_on_create,
            auto_start_on_restore,
        }): Parameters<UpdateProcessConfigRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        self.process_manager
            .update_process_config(id.clone(), auto_start_on_create, auto_start_on_restore)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let mut message = format!("Process '{id}' configuration updated");
        if let Some(value) = auto_start_on_create {
            message.push_str(&format!(" - auto_start_on_create set to {value}"));
        }
        if let Some(value) = auto_start_on_restore {
            message.push_str(&format!(" - auto_start_on_restore set to {value}"));
        }

        Ok(CallToolResult::success(vec![Content::text(message)]))
    }

    #[tool(description = "Update process attributes (command, args, env, cwd, and auto_start flags)")]
    async fn update_process(
        &self,
        Parameters(UpdateProcessRequest {
            id,
            command,
            args,
            env,
            cwd,
            auto_start_on_create,
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
                auto_start_on_create,
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
        if auto_start_on_create.is_some() {
            updates.push("auto_start_on_create");
        }
        if auto_start_on_restore.is_some() {
            updates.push("auto_start_on_restore");
        }

        let message = if updates.is_empty() {
            format!("Process '{}' - no attributes updated", id)
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
                            conclusion: run.conclusion.map(|c| format!("{:?}", c)),
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
                    format!("Failed to list CI runs: {}", e),
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
                    format!("Failed to get CI run details: {}", e),
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
                    format!("Failed to get CI failed logs: {}", e),
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
                    conclusion: run.conclusion.map(|c| format!("{:?}", c)),
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
                    format!("Failed to wait for CI completion: {}", e),
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
