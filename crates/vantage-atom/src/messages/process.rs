use crate::process::{OutputStream, ProcessFilter};
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateProcessRequest {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub auto_start_on_restore: bool, // サーバー起動時に自動起動
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartProcessRequest {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StopProcessRequest {
    pub grace_period_ms: Option<u64>,
}

// MCP tool用のリクエスト構造体（IDを含む）
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct McpStopProcessRequest {
    pub id: String,
    pub grace_period_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetProcessStatusRequest {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetProcessOutputRequest {
    pub id: String,
    pub stream: OutputStream,
    pub lines: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListProcessesRequest {
    pub filter: Option<ProcessFilter>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoveProcessRequest {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExportProcessesRequest {
    pub file_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ImportProcessesRequest {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateProcessConfigRequest {
    pub id: String,
    pub auto_start_on_restore: Option<bool>,
}

/// Request to update process attributes
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateProcessRequest {
    pub id: String,
    /// Optional: Update command
    pub command: Option<String>,
    /// Optional: Update args
    pub args: Option<Vec<String>>,
    /// Optional: Update environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Optional: Update working directory
    pub cwd: Option<String>,
    /// Optional: Update auto_start_on_restore flag
    pub auto_start_on_restore: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OpenWebConsoleRequest {
    /// Port to run the web console on (default: 12700)
    pub port: Option<u16>,
    /// Whether to open browser automatically (default: true)
    pub auto_open: Option<bool>,
}
