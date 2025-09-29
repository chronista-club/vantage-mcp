use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Process history record in database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessHistoryRecord {
    pub id: i64,
    pub process_id: String,
    pub name: String,
    pub command: String,
    pub args: Option<String>,  // JSON array
    pub env: Option<String>,   // JSON object
    pub cwd: Option<String>,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Process metrics record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessMetricsRecord {
    pub id: i64,
    pub process_id: String,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<i64>,
    pub disk_read: Option<i64>,
    pub disk_write: Option<i64>,
    pub network_rx: Option<i64>,
    pub network_tx: Option<i64>,
    pub recorded_at: DateTime<Utc>,
}

/// Process output log record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessOutputRecord {
    pub id: i64,
    pub process_id: String,
    pub stream_type: String,  // "stdout" or "stderr"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Clipboard record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ClipboardRecord {
    pub id: i64,
    pub key: String,
    pub content: String,
    pub content_type: String,  // "text", "json", "binary"
    pub metadata: Option<String>,  // JSON object for additional metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// System event record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemEventRecord {
    pub id: i64,
    pub event_type: String,  // "server_start", "server_stop", "process_crash", etc.
    pub description: String,
    pub details: Option<String>,  // JSON object for additional details
    pub severity: String,  // "info", "warning", "error"
    pub timestamp: DateTime<Utc>,
}

/// Process record (current configuration)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessRecord {
    pub id: i64,
    pub process_id: String,
    pub name: String,
    pub command: String,
    pub args: Option<String>,  // JSON array
    pub env: Option<String>,   // JSON object
    pub cwd: Option<String>,
    pub state: String,
    pub pid: Option<i32>,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub tags: Option<String>,  // JSON array
    pub auto_start_on_restore: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Process template record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProcessTemplateRecord {
    pub id: i64,
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub command: String,
    pub args: Option<String>,  // JSON array
    pub env: Option<String>,   // JSON object
    pub default_cwd: Option<String>,
    pub default_auto_start: bool,
    pub variables: Option<String>,  // JSON array
    pub tags: Option<String>,  // JSON array
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Settings record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SettingsRecord {
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
