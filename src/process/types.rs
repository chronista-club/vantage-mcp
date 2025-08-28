use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// プロセスの状態
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum ProcessState {
    NotStarted,
    Running {
        pid: u32,
        started_at: DateTime<Utc>,
    },
    Stopped {
        exit_code: Option<i32>,
        stopped_at: DateTime<Utc>,
    },
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
    },
}

/// プロセスの基本情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<PathBuf>,
    pub state: ProcessState,
    #[serde(default)]
    pub auto_start: bool,
}

/// プロセスの詳細ステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub info: ProcessInfo,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<u64>,
    pub uptime_seconds: Option<u64>,
}

/// 出力ストリームの種類
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum OutputStream {
    Stdout,
    Stderr,
    Both,
}

/// プロセスフィルター
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessFilter {
    pub state: Option<ProcessStateFilter>,
    pub name_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProcessStateFilter {
    Running,
    Stopped,
    Failed,
    All,
}

/// 再起動ポリシー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    Always,
    OnFailure { max_retries: u32 },
}
