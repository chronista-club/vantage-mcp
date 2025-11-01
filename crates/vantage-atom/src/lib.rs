use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
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
pub mod web;

pub use error::{VantageError, VantageResult};

use ci::CiMonitor;
use events::EventSystem;
use learning::LearningEngine;
use messages::*;
use process::ProcessManager;

#[derive(Clone)]
pub struct VantageServer {
    start_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    process_manager: ProcessManager,
    #[allow(dead_code)]
    event_system: Arc<EventSystem>,
    learning_engine: Arc<LearningEngine>,
    #[allow(dead_code)]
    ci_monitor: Arc<CiMonitor>,
    tool_router: ToolRouter<VantageServer>,
    db_connection: Option<Arc<vantage_persistence::DbConnection>>,
}

#[tool_router]
impl VantageServer {
    pub async fn new() -> anyhow::Result<Self> {
        tracing::info!("Initializing VantageServer");

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

        // DB接続を初期化（オプショナル）
        let db_connection = match vantage_persistence::DbConnection::new_default().await {
            Ok(conn) => {
                tracing::info!("SurrealDBに接続しました");

                // スキーマを自動適用
                let schema_manager = vantage_persistence::SchemaManager::new(conn.db());
                match schema_manager.apply_all().await {
                    Ok(_) => {
                        tracing::info!("データベーススキーマを適用しました");
                    }
                    Err(e) => {
                        tracing::error!(
                            "データベーススキーマの適用に失敗しました: {}. テンプレート機能が正常に動作しない可能性があります。\
                             SurrealDBの権限を確認し、データベースにアクセス可能であることを確認してください。",
                            e
                        );
                    }
                }

                Some(Arc::new(conn))
            }
            Err(e) => {
                tracing::warn!(
                    "SurrealDBへの接続に失敗しました: {}. テンプレート機能は利用できません。\
                     SurrealDBが起動しており、アクセス可能であることを確認してください。\
                     接続設定は以下の環境変数で設定できます: \
                     VANTAGE_DB_ENDPOINT, VANTAGE_DB_NAMESPACE, VANTAGE_DB_DATABASE, \
                     VANTAGE_DB_USERNAME, VANTAGE_DB_PASSWORD",
                    e
                );
                None
            }
        };

        tracing::info!("VantageServer initialization complete");
        Ok(Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager,
            event_system,
            learning_engine,
            ci_monitor,
            tool_router: Self::tool_router(),
            db_connection,
        })
    }

    pub fn set_process_manager(&mut self, manager: ProcessManager) {
        self.process_manager = manager;
    }

    /// Create VantageServer with existing ProcessManager (shares database)
    pub async fn with_process_manager(process_manager: ProcessManager) -> anyhow::Result<Self> {
        tracing::info!("Initializing VantageServer with existing ProcessManager");

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

        // CI監視を初期化（2回目の初期化）
        let ci_monitor_2 = Arc::new(CiMonitor::new(None, Some(30)));

        // DB接続を初期化（オプショナル）
        let db_connection = match vantage_persistence::DbConnection::new_default().await {
            Ok(conn) => {
                tracing::info!("SurrealDBに接続しました");

                // スキーマを自動適用
                let schema_manager = vantage_persistence::SchemaManager::new(conn.db());
                match schema_manager.apply_all().await {
                    Ok(_) => {
                        tracing::info!("データベーススキーマを適用しました");
                    }
                    Err(e) => {
                        tracing::error!(
                            "データベーススキーマの適用に失敗しました: {}. テンプレート機能が正常に動作しない可能性があります。\
                             SurrealDBの権限を確認し、データベースにアクセス可能であることを確認してください。",
                            e
                        );
                    }
                }

                Some(Arc::new(conn))
            }
            Err(e) => {
                tracing::warn!(
                    "SurrealDBへの接続に失敗しました: {}. テンプレート機能は利用できません。\
                     SurrealDBが起動しており、アクセス可能であることを確認してください。\
                     接続設定は以下の環境変数で設定できます: \
                     VANTAGE_DB_ENDPOINT, VANTAGE_DB_NAMESPACE, VANTAGE_DB_DATABASE, \
                     VANTAGE_DB_USERNAME, VANTAGE_DB_PASSWORD",
                    e
                );
                None
            }
        };

        tracing::info!("VantageServer initialization complete");

        Ok(Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager,
            event_system,
            learning_engine,
            ci_monitor: ci_monitor_2,
            tool_router: Self::tool_router(),
            db_connection,
        })
    }

    /// サーバー終了時の処理
    pub async fn shutdown(&self) -> std::result::Result<(), String> {
        tracing::info!("Shutting down VantageServer");

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

    // ========================================
    // Template Management Tools
    // ========================================

    #[tool(description = "Create a new process template for reusable configurations")]
    async fn create_template(
        &self,
        Parameters(request): Parameters<messages::template::CreateTemplateRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Creating template: {}", request.name);

        // DB接続の確認
        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available. Please ensure SurrealDB is running."
                .into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        // テンプレートリポジトリを作成
        let repo = vantage_persistence::TemplateRepository::new(db.db());

        // 名前の重複チェック
        if let Ok(Some(_)) = repo.get_by_name(&request.name).await {
            return Err(McpError {
                message: format!("Template with name '{}' already exists. Please use a different name or update the existing template.", request.name).into(),
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                data: None,
            });
        }

        // カテゴリの変換
        let category = request
            .category
            .as_ref()
            .map(|c| match c.to_lowercase().as_str() {
                "database" => vantage_persistence::TemplateCategory::Database,
                "web_server" | "webserver" => vantage_persistence::TemplateCategory::WebServer,
                "build_tool" | "buildtool" => vantage_persistence::TemplateCategory::BuildTool,
                "script" => vantage_persistence::TemplateCategory::Script,
                _ => vantage_persistence::TemplateCategory::Other,
            })
            .unwrap_or(vantage_persistence::TemplateCategory::Other);

        // Templateオブジェクトを作成
        let mut template =
            vantage_persistence::Template::new(request.name.clone(), request.command.clone());
        template.description = request.description;
        template.category = category;
        template.args = request.args.unwrap_or_default();
        template.env = request.env.unwrap_or_default();
        template.cwd = request.cwd;
        template.tags = request.tags.unwrap_or_default();

        // データベースに保存
        let created = repo.create(template).await.map_err(|e| McpError {
            message: format!("Failed to create template: {}", e).into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let response = serde_json::json!({
            "success": true,
            "template_id": created.id.as_ref().map(|id| id.to_string()),
            "name": created.name,
            "message": format!("Template '{}' created successfully", created.name)
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "List all process templates, optionally filtered by category or tag")]
    async fn list_templates(
        &self,
        Parameters(request): Parameters<messages::template::ListTemplatesRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Listing templates");

        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available".into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let repo = vantage_persistence::TemplateRepository::new(db.db());

        let templates = if let Some(category_str) = request.category {
            let category = match category_str.to_lowercase().as_str() {
                "database" => vantage_persistence::TemplateCategory::Database,
                "web_server" | "webserver" => vantage_persistence::TemplateCategory::WebServer,
                "build_tool" | "buildtool" => vantage_persistence::TemplateCategory::BuildTool,
                "script" => vantage_persistence::TemplateCategory::Script,
                _ => vantage_persistence::TemplateCategory::Other,
            };
            repo.list_by_category(category).await
        } else if let Some(tag) = request.tag {
            repo.search_by_tag(&tag).await
        } else {
            repo.list().await
        }
        .map_err(|e| McpError {
            message: format!("Failed to list templates: {}", e).into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let template_list: Vec<_> = templates
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.id.as_ref().map(|id| id.to_string()),
                    "name": t.name,
                    "description": t.description,
                    "category": format!("{:?}", t.category),
                    "command": t.command,
                    "tags": t.tags,
                    "use_count": t.use_count,
                })
            })
            .collect();

        let response = serde_json::json!({
            "templates": template_list,
            "count": templates.len(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Get detailed information about a specific template by ID or name")]
    async fn get_template(
        &self,
        Parameters(request): Parameters<messages::template::GetTemplateRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available".into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let repo = vantage_persistence::TemplateRepository::new(db.db());

        let template = if let Some(id) = request.id {
            tracing::info!("Getting template by ID: {}", id);
            repo.get(&id).await
        } else if let Some(name) = request.name {
            tracing::info!("Getting template by name: {}", name);
            repo.get_by_name(&name).await
        } else {
            return Err(McpError {
                message: "Either 'id' or 'name' must be provided".into(),
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                data: None,
            });
        }
        .map_err(|e| McpError {
            message: format!("Failed to get template: {}", e).into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?
        .ok_or_else(|| McpError {
            message: "Template not found".into(),
            code: rmcp::model::ErrorCode::INVALID_PARAMS,
            data: None,
        })?;

        let response = serde_json::json!({
            "id": template.id.as_ref().map(|id| id.to_string()),
            "name": template.name,
            "description": template.description,
            "category": format!("{:?}", template.category),
            "command": template.command,
            "args": template.args,
            "env": template.env,
            "cwd": template.cwd,
            "tags": template.tags,
            "use_count": template.use_count,
            "created_at": template.created_at,
            "updated_at": template.updated_at,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Update an existing template")]
    async fn update_template(
        &self,
        Parameters(request): Parameters<messages::template::UpdateTemplateRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::info!("Updating template: {}", request.id);

        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available".into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let repo = vantage_persistence::TemplateRepository::new(db.db());

        // 既存のテンプレートを取得
        let mut template = repo
            .get(&request.id)
            .await
            .map_err(|e| McpError {
                message: format!("Failed to get template: {}", e).into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?
            .ok_or_else(|| McpError {
                message: format!("Template with ID '{}' not found", request.id).into(),
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                data: None,
            })?;

        // 更新
        if let Some(name) = request.name {
            template.name = name;
        }
        if let Some(command) = request.command {
            template.command = command;
        }
        if let Some(description) = request.description {
            template.description = Some(description);
        }
        if let Some(category_str) = request.category {
            template.category = match category_str.to_lowercase().as_str() {
                "database" => vantage_persistence::TemplateCategory::Database,
                "web_server" | "webserver" => vantage_persistence::TemplateCategory::WebServer,
                "build_tool" | "buildtool" => vantage_persistence::TemplateCategory::BuildTool,
                "script" => vantage_persistence::TemplateCategory::Script,
                _ => vantage_persistence::TemplateCategory::Other,
            };
        }
        if let Some(tags) = request.tags {
            template.tags = tags;
        }
        if let Some(args) = request.args {
            template.args = args;
        }
        if let Some(env) = request.env {
            template.env = env;
        }
        if let Some(cwd) = request.cwd {
            template.cwd = Some(cwd);
        }

        let updated = repo
            .update(&request.id, template)
            .await
            .map_err(|e| McpError {
                message: format!("Failed to update template: {}", e).into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        let response = serde_json::json!({
            "success": true,
            "template_id": updated.id.as_ref().map(|id| id.to_string()),
            "name": updated.name,
            "message": "Template updated successfully"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Delete a template by ID or name")]
    async fn delete_template(
        &self,
        Parameters(request): Parameters<messages::template::DeleteTemplateRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available".into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let repo = vantage_persistence::TemplateRepository::new(db.db());

        let (id, name) = if let Some(id) = request.id {
            tracing::info!("Deleting template by ID: {}", id);
            (id, None)
        } else if let Some(name) = request.name.clone() {
            tracing::info!("Deleting template by name: {}", name);
            // 名前からIDを取得
            let template = repo
                .get_by_name(&name)
                .await
                .map_err(|e| McpError {
                    message: format!("Failed to get template: {}", e).into(),
                    code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                    data: None,
                })?
                .ok_or_else(|| McpError {
                    message: format!("Template '{}' not found", name).into(),
                    code: rmcp::model::ErrorCode::INVALID_PARAMS,
                    data: None,
                })?;
            let id = template
                .id
                .as_ref()
                .ok_or_else(|| McpError {
                    message: "Template has no ID".into(),
                    code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                    data: None,
                })?
                .to_string();
            (id, Some(name))
        } else {
            return Err(McpError {
                message: "Either 'id' or 'name' must be provided".into(),
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                data: None,
            });
        };

        repo.delete(&id).await.map_err(|e| McpError {
            message: format!("Failed to delete template: {}", e).into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let response = serde_json::json!({
            "success": true,
            "message": format!("Template '{}' deleted successfully", name.unwrap_or_else(|| id.clone()))
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Create a new process from a template with optional overrides")]
    async fn create_process_from_template(
        &self,
        Parameters(request): Parameters<messages::template::CreateProcessFromTemplateRequest>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let db = self.db_connection.as_ref().ok_or_else(|| McpError {
            message: "Database connection not available".into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?;

        let repo = vantage_persistence::TemplateRepository::new(db.db());

        // テンプレートを取得
        let template = if let Some(id) = request.template_id {
            tracing::info!("Getting template by ID: {}", id);
            repo.get(&id).await
        } else if let Some(name) = request.template_name {
            tracing::info!("Getting template by name: {}", name);
            repo.get_by_name(&name).await
        } else {
            return Err(McpError {
                message: "Either 'template_id' or 'template_name' must be provided".into(),
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                data: None,
            });
        }
        .map_err(|e| McpError {
            message: format!("Failed to get template: {}", e).into(),
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            data: None,
        })?
        .ok_or_else(|| McpError {
            message: "Template not found".into(),
            code: rmcp::model::ErrorCode::INVALID_PARAMS,
            data: None,
        })?;

        // プロセスを作成（オーバーライドを適用）
        let command = template.command.clone();
        let args = request.override_args.unwrap_or(template.args.clone());
        let env = request.override_env.unwrap_or(template.env.clone());
        let cwd = request
            .override_cwd
            .or(template.cwd.clone())
            .map(std::path::PathBuf::from);

        // ProcessManager経由でプロセスを作成
        self.process_manager
            .create_process(
                request.process_id.clone(),
                command,
                args,
                env,
                cwd,
                request.auto_start.unwrap_or(false),
            )
            .await
            .map_err(|e| McpError {
                message: format!("Failed to create process: {}", e).into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;

        // 使用回数を更新
        let template_id = template
            .id
            .as_ref()
            .ok_or_else(|| McpError {
                message: "Template has no ID".into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?
            .to_string();

        if let Err(e) = repo.increment_use_count(&template_id).await {
            tracing::warn!("Failed to increment template use count: {}", e);
        }

        let response = serde_json::json!({
            "success": true,
            "process_id": request.process_id,
            "template_name": template.name,
            "message": format!("Process '{}' created from template '{}'", request.process_id, template.name)
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap(),
        )]))
    }

    #[tool(description = "Open the Vantage web console in your browser")]
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
                if auto_open && let Err(e) = open::that(&url) {
                    tracing::warn!("Failed to open browser: {}", e);
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Web console is already running at {url}. Please open it manually."
                    ))]));
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
                    "Web console is not running. Please start Vantage with web mode:\n\
                     \n\
                     vantagemcp --web-only --web-port {port}\n\
                     \n\
                     Or use the default port:\n\
                     vantagemcp --web-only\n\
                     \n\
                     The web console will be available at {url}"
                ))]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for VantageServer {
    fn get_info(&self) -> ServerInfo {
        tracing::info!("MCP client requesting server info");
        let info = ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "vantage-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("Vantage MCP".to_string()),
                website_url: Some("https://github.com/chronista-club/vantage-mcp".to_string()),
                icons: None,
            },
            instructions: Some(
                "Vantage MCP - A powerful process management server for Claude Code via MCP."
                    .to_string(),
            ),
        };
        tracing::debug!("Returning server info: {:?}", info.server_info);
        info
    }
}
