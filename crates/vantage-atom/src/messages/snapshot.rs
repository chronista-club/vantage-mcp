use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request to export processes to YAML format
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExportYamlRequest {
    /// Optional file path. If not provided, uses default location
    pub file_path: Option<String>,
    /// Export only processes with auto_start_on_restore flag set to true
    pub only_auto_start: bool,
}

/// Request to import processes from YAML format
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ImportYamlRequest {
    /// File path to import from
    pub file_path: String,
}

/// Request to create a snapshot
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSnapshotRequest {
    /// Optional file path. If not provided, uses default location
    pub file_path: Option<String>,
    /// Snapshot format (yaml or surql)
    pub format: SnapshotFormat,
}

/// Request to restore from a snapshot
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RestoreSnapshotRequest {
    /// Optional file path. If not provided, uses default location
    pub file_path: Option<String>,
    /// Snapshot format (yaml or surql)
    pub format: SnapshotFormat,
}

/// Snapshot format
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotFormat {
    Yaml,
    Surql,
}
