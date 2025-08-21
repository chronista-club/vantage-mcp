use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod messages;
pub mod persistence;
pub mod process;
#[cfg(feature = "web")]
pub mod web;

use messages::*;
use process::ProcessManager;

#[derive(Clone)]
pub struct IchimiServer {
    start_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    process_manager: ProcessManager,
    tool_router: ToolRouter<IchimiServer>,
}

#[tool_router]
impl IchimiServer {
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(Mutex::new(chrono::Utc::now())),
            process_manager: ProcessManager::new(),
            tool_router: Self::tool_router(),
        }
    }
    
    pub fn set_process_manager(&mut self, manager: ProcessManager) {
        self.process_manager = manager;
    }

    #[tool(description = "Echo the input message back")]
    fn echo(
        &self,
        Parameters(EchoRequest { message }): Parameters<EchoRequest>,
    ) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Echo: {}",
            message
        ))]))
    }

    #[tool(description = "Simple ping/pong health check")]
    fn ping(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("pong")]))
    }

    #[tool(description = "Get the current server status")]
    async fn get_status(&self) -> Result<CallToolResult, McpError> {
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
        Parameters(CreateProcessRequest { id, command, args, env, cwd }): Parameters<CreateProcessRequest>,
    ) -> Result<CallToolResult, McpError> {
        let cwd_path = cwd.map(std::path::PathBuf::from);
        
        self.process_manager
            .create_process(id.clone(), command, args, env, cwd_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{}' created successfully",
            id
        ))]))
    }

    #[tool(description = "Start a registered process")]
    async fn start_process(
        &self,
        Parameters(StartProcessRequest { id }): Parameters<StartProcessRequest>,
    ) -> Result<CallToolResult, McpError> {
        let pid = self.process_manager
            .start_process(id.clone())
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{}' started with PID {}",
            id, pid
        ))]))
    }

    #[tool(description = "Stop a running process")]
    async fn stop_process(
        &self,
        Parameters(StopProcessRequest { id, grace_period_ms }): Parameters<StopProcessRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.process_manager
            .stop_process(id.clone(), grace_period_ms)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{}' stopped successfully",
            id
        ))]))
    }

    #[tool(description = "Get process status and metrics")]
    async fn get_process_status(
        &self,
        Parameters(GetProcessStatusRequest { id }): Parameters<GetProcessStatusRequest>,
    ) -> Result<CallToolResult, McpError> {
        let status = self.process_manager
            .get_process_status(id)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        let json = serde_json::to_string_pretty(&status)
            .map_err(|e| McpError {
                message: format!("Failed to serialize status: {}", e).into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Get process output (stdout/stderr)")]
    async fn get_process_output(
        &self,
        Parameters(GetProcessOutputRequest { id, stream, lines }): Parameters<GetProcessOutputRequest>,
    ) -> Result<CallToolResult, McpError> {
        let output = self.process_manager
            .get_process_output(id, stream, lines)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(output.join("\n"))]))
    }

    #[tool(description = "List all managed processes")]
    async fn list_processes(
        &self,
        Parameters(ListProcessesRequest { filter }): Parameters<ListProcessesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let processes = self.process_manager
            .list_processes(filter)
            .await;
        
        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| McpError {
                message: format!("Failed to serialize processes: {}", e).into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Remove a process from management")]
    async fn remove_process(
        &self,
        Parameters(RemoveProcessRequest { id }): Parameters<RemoveProcessRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.process_manager
            .remove_process(id.clone())
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Process '{}' removed successfully",
            id
        ))]))
    }
    
    #[tool(description = "Export all processes to a surql file for backup/persistence")]
    async fn export_processes(
        &self,
        Parameters(ExportProcessesRequest { file_path }): Parameters<ExportProcessesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let path = self.process_manager
            .export_processes(file_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Processes exported successfully to {}",
            path
        ))]))
    }
    
    #[tool(description = "Import processes from a surql file")]
    async fn import_processes(
        &self,
        Parameters(ImportProcessesRequest { file_path }): Parameters<ImportProcessesRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.process_manager
            .import_processes(&file_path)
            .await
            .map_err(|e| McpError {
                message: e.into(),
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                data: None,
            })?;
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Processes imported successfully from {}",
            file_path
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for IchimiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "ichimi-server".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some(
                "Ichimi Server - A powerful process management server for Claude Code via MCP."
                    .to_string(),
            ),
        }
    }
}