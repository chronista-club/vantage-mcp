use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::types::{ProcessInfo, ProcessState, ProcessStatus};

/// YAML snapshot format version
const SNAPSHOT_VERSION: &str = "1.0";

/// Root structure for YAML snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: SnapshotMetadata,
    pub processes: Vec<ProcessSnapshot>,
}

/// Metadata for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub ichimi_version: String,
    pub total_processes: usize,
    pub auto_start_processes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Individual process snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub auto_start_on_restore: bool,
    pub state: ProcessStateSnapshot,
    pub tags: Vec<String>,
}

/// Process state in snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum ProcessStateSnapshot {
    #[serde(rename = "not_started")]
    NotStarted,
    #[serde(rename = "running")]
    Running {
        pid: u32,
        started_at: DateTime<Utc>,
    },
    #[serde(rename = "stopped")]
    Stopped {
        exit_code: Option<i32>,
        stopped_at: DateTime<Utc>,
    },
    #[serde(rename = "failed")]
    Failed {
        error: String,
        failed_at: DateTime<Utc>,
    },
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(processes: Vec<ProcessSnapshot>) -> Self {
        let auto_start_count = processes
            .iter()
            .filter(|p| p.auto_start_on_restore)
            .count();

        Self {
            version: SNAPSHOT_VERSION.to_string(),
            timestamp: Utc::now(),
            metadata: SnapshotMetadata {
                ichimi_version: env!("CARGO_PKG_VERSION").to_string(),
                total_processes: processes.len(),
                auto_start_processes: auto_start_count,
                hostname: hostname::get()
                    .ok()
                    .and_then(|h| h.to_str().map(|s| s.to_string())),
                description: None,
            },
            processes,
        }
    }

    /// Create a snapshot with only auto-start processes
    pub fn new_auto_start_only(processes: Vec<ProcessSnapshot>) -> Self {
        let filtered_processes: Vec<ProcessSnapshot> = processes
            .into_iter()
            .filter(|p| p.auto_start_on_restore)
            .collect();

        Self::new(filtered_processes)
    }

    /// Export snapshot to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize snapshot to YAML")
    }

    /// Import snapshot from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to deserialize snapshot from YAML")
    }

    /// Save snapshot to file
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create snapshot directory")?;
        }

        let yaml = self.to_yaml()?;
        tokio::fs::write(path, yaml)
            .await
            .context("Failed to write snapshot file")?;

        Ok(())
    }

    /// Load snapshot from file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let yaml = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read snapshot file")?;

        Self::from_yaml(&yaml)
    }

    /// Validate snapshot version compatibility
    pub fn is_compatible(&self) -> bool {
        // For now, only accept exact version match
        // In future, we can implement version migration logic
        self.version == SNAPSHOT_VERSION
    }
}

impl From<&ProcessInfo> for ProcessSnapshot {
    fn from(info: &ProcessInfo) -> Self {
        Self {
            id: info.process_id.clone(),
            name: info.name.clone(),
            command: info.command.clone(),
            args: info.args.clone(),
            env: info.env.clone(),
            cwd: info.cwd.clone(),
            auto_start_on_restore: info.auto_start_on_restore,
            state: ProcessStateSnapshot::from(&info.status),
            tags: info.tags.clone(),
        }
    }
}

impl From<&ProcessStatus> for ProcessStateSnapshot {
    fn from(status: &ProcessStatus) -> Self {
        match status.state {
            ProcessState::NotStarted => ProcessStateSnapshot::NotStarted,
            ProcessState::Running => {
                if let (Some(pid), Some(started_at)) = (status.pid, status.started_at) {
                    ProcessStateSnapshot::Running { pid, started_at }
                } else {
                    ProcessStateSnapshot::NotStarted
                }
            }
            ProcessState::Stopped => {
                if let Some(stopped_at) = status.stopped_at {
                    ProcessStateSnapshot::Stopped {
                        exit_code: status.exit_code,
                        stopped_at,
                    }
                } else {
                    ProcessStateSnapshot::NotStarted
                }
            }
            ProcessState::Failed => {
                if let (Some(error), Some(stopped_at)) =
                    (status.error.clone(), status.stopped_at) {
                    ProcessStateSnapshot::Failed {
                        error,
                        failed_at: stopped_at,
                    }
                } else {
                    ProcessStateSnapshot::NotStarted
                }
            }
        }
    }
}

impl ProcessSnapshot {
    /// Convert snapshot to ProcessInfo
    pub fn to_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: None, // Will be set by database
            process_id: self.id.clone(),
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.clone(),
            status: self.state.to_process_status(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: self.tags.clone(),
            auto_start_on_restore: self.auto_start_on_restore,
        }
    }
}

impl ProcessStateSnapshot {
    /// Convert snapshot state to ProcessStatus
    pub fn to_process_status(&self) -> ProcessStatus {
        match self {
            ProcessStateSnapshot::NotStarted => ProcessStatus {
                state: ProcessState::NotStarted,
                pid: None,
                exit_code: None,
                started_at: None,
                stopped_at: None,
                error: None,
            },
            ProcessStateSnapshot::Running { pid, started_at } => ProcessStatus {
                state: ProcessState::Running,
                pid: Some(*pid),
                exit_code: None,
                started_at: Some(*started_at),
                stopped_at: None,
                error: None,
            },
            ProcessStateSnapshot::Stopped { exit_code, stopped_at } => ProcessStatus {
                state: ProcessState::Stopped,
                pid: None,
                exit_code: *exit_code,
                started_at: None,
                stopped_at: Some(*stopped_at),
                error: None,
            },
            ProcessStateSnapshot::Failed { error, failed_at } => ProcessStatus {
                state: ProcessState::Failed,
                pid: None,
                exit_code: None,
                started_at: None,
                stopped_at: Some(*failed_at),
                error: Some(error.clone()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot::new(vec![
            ProcessSnapshot {
                id: "test-process".to_string(),
                name: "Test Process".to_string(),
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                env: HashMap::new(),
                cwd: None,
                auto_start_on_restore: true,
                state: ProcessStateSnapshot::NotStarted,
                tags: vec![],
            },
        ]);

        let yaml = snapshot.to_yaml().unwrap();
        assert!(yaml.contains("version: '1.0'"));
        assert!(yaml.contains("test-process"));
        assert!(yaml.contains("auto_start_on_restore: true"));

        let deserialized = Snapshot::from_yaml(&yaml).unwrap();
        assert_eq!(deserialized.version, SNAPSHOT_VERSION);
        assert_eq!(deserialized.processes.len(), 1);
        assert_eq!(deserialized.processes[0].id, "test-process");
    }

    #[test]
    fn test_auto_start_filter() {
        let processes = vec![
            ProcessSnapshot {
                id: "auto-1".to_string(),
                name: "Auto 1".to_string(),
                command: "cmd1".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
                auto_start_on_restore: true,
                state: ProcessStateSnapshot::NotStarted,
                tags: vec![],
            },
            ProcessSnapshot {
                id: "manual-1".to_string(),
                name: "Manual 1".to_string(),
                command: "cmd2".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
                auto_start_on_restore: false,
                state: ProcessStateSnapshot::NotStarted,
                tags: vec![],
            },
            ProcessSnapshot {
                id: "auto-2".to_string(),
                name: "Auto 2".to_string(),
                command: "cmd3".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
                auto_start_on_restore: true,
                state: ProcessStateSnapshot::NotStarted,
                tags: vec![],
            },
        ];

        let snapshot = Snapshot::new_auto_start_only(processes);
        assert_eq!(snapshot.processes.len(), 2);
        assert!(snapshot.processes.iter().all(|p| p.auto_start_on_restore));
        assert_eq!(snapshot.metadata.auto_start_processes, 2);
    }
}