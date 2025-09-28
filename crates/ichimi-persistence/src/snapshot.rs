use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;

use crate::kdl_serde::KdlSnapshot;
use crate::types::{ProcessInfo, ProcessState, ProcessStatus};

/// Snapshot format for process persistence (wrapper around KDL implementation)
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub hostname: Option<String>,
    pub processes: Vec<ProcessSnapshot>,
}

/// Individual process in snapshot
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub auto_start: bool,
    pub state: ProcessStateSnapshot,
    pub tags: Vec<String>,
}

/// Process state in snapshot
#[derive(Debug, Clone)]
pub enum ProcessStateSnapshot {
    NotStarted,
    Running { pid: u32, started_at: DateTime<Utc> },
    Stopped { exit_code: Option<i32>, stopped_at: DateTime<Utc> },
    Failed { error: String, failed_at: DateTime<Utc> },
}

impl Snapshot {
    /// Create a new snapshot from processes
    pub fn new(processes: Vec<ProcessInfo>) -> Self {
        let kdl_snapshot = KdlSnapshot::from_processes(processes.clone());
        Self::from_kdl(kdl_snapshot, processes)
    }

    /// Filter to only auto-start processes
    pub fn filter_auto_start(mut self) -> Self {
        self.processes.retain(|p| p.auto_start);
        self
    }

    /// Export to KDL format
    pub fn to_kdl(&self) -> String {
        let processes: Vec<ProcessInfo> = self.processes.iter().map(|p| p.to_process_info()).collect();
        let kdl_snapshot = KdlSnapshot::from_processes(processes);
        kdl_snapshot.to_kdl_string().unwrap_or_else(|e| {
            eprintln!("Error generating KDL: {}", e);
            String::new()
        })
    }

    /// Parse from KDL format
    pub fn from_kdl_str(content: &str) -> Result<Self> {
        let kdl_snapshot = KdlSnapshot::from_kdl_string(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse KDL snapshot: {}", e))?;

        let processes: Vec<ProcessInfo> = kdl_snapshot.processes
            .iter()
            .map(|p| p.to_process_info())
            .collect();

        Ok(Self::from_kdl(kdl_snapshot, processes))
    }

    /// Internal conversion from KDL snapshot
    fn from_kdl(kdl: KdlSnapshot, processes: Vec<ProcessInfo>) -> Self {
        let timestamp = DateTime::parse_from_rfc3339(&kdl.meta.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Self {
            version: kdl.meta.version,
            timestamp,
            hostname: kdl.meta.hostname,
            processes: processes.iter().map(ProcessSnapshot::from).collect(),
        }
    }

    /// Save to file
    pub async fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create directory")?;
        }

        let content = self.to_kdl();
        tokio::fs::write(path, content)
            .await
            .context("Failed to write snapshot")?;

        Ok(())
    }

    /// Load from file
    pub async fn load(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read snapshot")?;

        Self::from_kdl_str(&content)
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
            auto_start: info.auto_start_on_restore,
            state: ProcessStateSnapshot::from(&info.status),
            tags: info.tags.clone(),
        }
    }
}

impl ProcessSnapshot {
    /// Convert to ProcessInfo
    pub fn to_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: None,
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
            auto_start_on_restore: self.auto_start,
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
                if let (Some(error), Some(stopped_at)) = (&status.error, status.stopped_at) {
                    ProcessStateSnapshot::Failed {
                        error: error.clone(),
                        failed_at: stopped_at,
                    }
                } else {
                    ProcessStateSnapshot::NotStarted
                }
            }
        }
    }
}

impl ProcessStateSnapshot {
    /// Convert to ProcessStatus
    fn to_process_status(&self) -> ProcessStatus {
        match self {
            ProcessStateSnapshot::NotStarted => ProcessStatus::default(),
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

    #[tokio::test]
    async fn test_snapshot_kdl_roundtrip() {
        let process = ProcessInfo {
            id: None,
            process_id: "test-server".to_string(),
            name: "Test Server".to_string(),
            command: "python".to_string(),
            args: vec!["-m".to_string(), "http.server".to_string()],
            env: HashMap::from([
                ("PORT".to_string(), "8000".to_string()),
                ("DEBUG".to_string(), "true".to_string()),
            ]),
            cwd: Some("/tmp".to_string()),
            status: ProcessStatus::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["web".to_string()],
            auto_start_on_restore: true,
        };

        let snapshot = Snapshot::new(vec![process]);
        let kdl = snapshot.to_kdl();

        // Check KDL content
        assert!(kdl.contains("process \"test-server\""));
        assert!(kdl.contains("name \"Test Server\""));
        assert!(kdl.contains("command \"python\""));
        assert!(kdl.contains("auto_start #true"));

        // Parse back
        let parsed = Snapshot::from_kdl_str(&kdl).unwrap();
        assert_eq!(parsed.processes.len(), 1);
        assert_eq!(parsed.processes[0].id, "test-server");
        assert_eq!(parsed.processes[0].name, "Test Server");
        assert!(parsed.processes[0].auto_start);
    }
}