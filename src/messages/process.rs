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
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartProcessRequest {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StopProcessRequest {
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
    pub auto_start: Option<bool>,
}
