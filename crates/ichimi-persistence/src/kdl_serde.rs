use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{ProcessInfo, ProcessState, ProcessStatus};

/// KDL snapshot root document with serde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdlSnapshot {
    pub meta: KdlMeta,
    pub processes: Vec<KdlProcess>,
}

/// Metadata block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdlMeta {
    pub version: String,
    pub timestamp: String,
    pub hostname: Option<String>,
}

/// Process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdlProcess {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub auto_start: bool,
    pub tags: Vec<String>,
    pub env: HashMap<String, String>,
    pub state: Option<KdlProcessState>,
}

/// Process state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdlProcessState {
    pub state_type: String,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

impl KdlSnapshot {
    /// Create from ProcessInfo list
    pub fn from_processes(processes: Vec<ProcessInfo>) -> Self {
        Self {
            meta: KdlMeta {
                version: "1.0".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                hostname: hostname::get()
                    .ok()
                    .and_then(|h| h.to_str().map(|s| s.to_string())),
            },
            processes: processes.into_iter().map(KdlProcess::from).collect(),
        }
    }

    /// Convert to KDL string using serde-kdl
    pub fn to_kdl_string(&self) -> Result<String, String> {
        // First create a KDL document
        let mut doc = kdl::KdlDocument::new();

        // Add meta node
        let mut meta_node = kdl::KdlNode::new("meta");
        meta_node.insert("version", self.meta.version.clone());
        meta_node.insert("timestamp", self.meta.timestamp.clone());
        if let Some(hostname) = &self.meta.hostname {
            meta_node.insert("hostname", hostname.clone());
        }
        doc.nodes_mut().push(meta_node);

        // Add process nodes
        for process in &self.processes {
            let mut process_node = kdl::KdlNode::new("process");
            process_node.push(kdl::KdlEntry::new(process.id.clone()));

            process_node.insert("name", process.name.clone());
            process_node.insert("command", process.command.clone());

            // Add args as properties
            for arg in &process.args {
                process_node.insert("args", arg.clone());
            }

            if let Some(cwd) = &process.cwd {
                process_node.insert("cwd", cwd.clone());
            }

            if process.auto_start {
                process_node.insert("auto_start", true);
            }

            // Add tags
            for tag in &process.tags {
                process_node.insert("tag", tag.clone());
            }

            // Add env block
            if !process.env.is_empty() {
                let mut env_node = kdl::KdlNode::new("env");
                let env_doc = env_node.ensure_children();
                for (key, value) in &process.env {
                    let mut var_node = kdl::KdlNode::new("var");
                    var_node.push(kdl::KdlEntry::new(key.clone()));
                    var_node.push(kdl::KdlEntry::new(value.clone()));
                    env_doc.nodes_mut().push(var_node);
                }
                process_node.ensure_children().nodes_mut().push(env_node);
            }

            // Add state block
            if let Some(state) = &process.state {
                let mut state_node = kdl::KdlNode::new("state");
                state_node.push(kdl::KdlEntry::new(state.state_type.clone()));

                if let Some(pid) = state.pid {
                    state_node.insert("pid", pid as i64);
                }
                if let Some(started_at) = &state.started_at {
                    state_node.insert("started_at", started_at.clone());
                }
                if let Some(stopped_at) = &state.stopped_at {
                    state_node.insert("stopped_at", stopped_at.clone());
                }
                if let Some(exit_code) = state.exit_code {
                    state_node.insert("exit_code", exit_code as i64);
                }
                if let Some(error) = &state.error {
                    state_node.insert("error", error.clone());
                }

                process_node.ensure_children().nodes_mut().push(state_node);
            }

            doc.nodes_mut().push(process_node);
        }

        // Convert to string with comments
        let mut kdl_string = String::new();
        kdl_string.push_str("// Ichimi Process Snapshot\n");
        kdl_string.push_str(&format!("// 生成日時: {}\n\n", self.meta.timestamp));
        kdl_string.push_str(&doc.to_string());

        Ok(kdl_string)
    }

    /// Parse from KDL string
    pub fn from_kdl_string(content: &str) -> Result<Self, String> {
        // Remove comments for parsing
        let clean_content: String = content
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        let doc = clean_content
            .parse::<kdl::KdlDocument>()
            .map_err(|e| format!("Failed to parse KDL: {e}"))?;

        let mut meta = KdlMeta {
            version: "1.0".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            hostname: None,
        };

        let mut processes = Vec::new();

        for node in doc.nodes() {
            match node.name().value() {
                "meta" => {
                    if let Some(version) = node.get("version").and_then(|e| e.value().as_string()) {
                        meta.version = version.to_string();
                    }
                    if let Some(timestamp) =
                        node.get("timestamp").and_then(|e| e.value().as_string())
                    {
                        meta.timestamp = timestamp.to_string();
                    }
                    if let Some(hostname) = node.get("hostname").and_then(|e| e.value().as_string())
                    {
                        meta.hostname = Some(hostname.to_string());
                    }
                }
                "process" => {
                    let id = node
                        .get(0)
                        .and_then(|e| e.value().as_string())
                        .ok_or("Process ID not found")?
                        .to_string();

                    let name = node
                        .get("name")
                        .and_then(|e| e.value().as_string())
                        .unwrap_or(&id)
                        .to_string();

                    let command = node
                        .get("command")
                        .and_then(|e| e.value().as_string())
                        .unwrap_or("")
                        .to_string();

                    // Collect args
                    let args: Vec<String> = node
                        .entries()
                        .iter()
                        .filter(|e| e.name().map(|n| n.value()) == Some("args"))
                        .filter_map(|e| e.value().as_string().map(String::from))
                        .collect();

                    let cwd = node
                        .get("cwd")
                        .and_then(|e| e.value().as_string())
                        .map(String::from);

                    let auto_start = node
                        .get("auto_start")
                        .and_then(|e| e.value().as_bool())
                        .unwrap_or(false);

                    // Collect tags
                    let tags: Vec<String> = node
                        .entries()
                        .iter()
                        .filter(|e| e.name().map(|n| n.value()) == Some("tag"))
                        .filter_map(|e| e.value().as_string().map(String::from))
                        .collect();

                    // Parse env block
                    let mut env = HashMap::new();
                    if let Some(children) = node.children() {
                        for child in children.nodes() {
                            if child.name().value() == "env" {
                                if let Some(env_children) = child.children() {
                                    for var_node in env_children.nodes() {
                                        if var_node.name().value() == "var" {
                                            if let (Some(key), Some(value)) = (
                                                var_node.get(0).and_then(|e| e.value().as_string()),
                                                var_node.get(1).and_then(|e| e.value().as_string()),
                                            ) {
                                                env.insert(key.to_string(), value.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Parse state block
                    let mut state = None;
                    if let Some(children) = node.children() {
                        for child in children.nodes() {
                            if child.name().value() == "state" {
                                let state_type = child
                                    .get(0)
                                    .and_then(|e| e.value().as_string())
                                    .unwrap_or("unknown")
                                    .to_string();

                                state = Some(KdlProcessState {
                                    state_type,
                                    pid: child
                                        .get("pid")
                                        .and_then(|e| e.value().as_i64())
                                        .map(|v| v as u32),
                                    started_at: child
                                        .get("started_at")
                                        .and_then(|e| e.value().as_string())
                                        .map(String::from),
                                    stopped_at: child
                                        .get("stopped_at")
                                        .and_then(|e| e.value().as_string())
                                        .map(String::from),
                                    exit_code: child
                                        .get("exit_code")
                                        .and_then(|e| e.value().as_i64())
                                        .map(|v| v as i32),
                                    error: child
                                        .get("error")
                                        .and_then(|e| e.value().as_string())
                                        .map(String::from),
                                });
                            }
                        }
                    }

                    processes.push(KdlProcess {
                        id,
                        name,
                        command,
                        args,
                        cwd,
                        auto_start,
                        tags,
                        env,
                        state,
                    });
                }
                _ => {} // Ignore unknown nodes
            }
        }

        Ok(Self { meta, processes })
    }
}

impl From<ProcessInfo> for KdlProcess {
    fn from(info: ProcessInfo) -> Self {
        let state = KdlProcessState::from_status(&info.status);

        Self {
            id: info.process_id,
            name: info.name,
            command: info.command,
            args: info.args,
            cwd: info.cwd,
            auto_start: info.auto_start_on_restore,
            tags: info.tags,
            env: info.env,
            state,
        }
    }
}

impl KdlProcess {
    /// Convert to ProcessInfo
    pub fn to_process_info(&self) -> ProcessInfo {
        let status = self
            .state
            .as_ref()
            .map(|s| s.to_process_status())
            .unwrap_or_default();

        ProcessInfo {
            id: None,
            process_id: self.id.clone(),
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.clone(),
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: self.tags.clone(),
            auto_start_on_restore: self.auto_start,
        }
    }
}

impl KdlProcessState {
    /// Create from ProcessStatus
    pub fn from_status(status: &ProcessStatus) -> Option<Self> {
        match status.state {
            ProcessState::NotStarted => None,
            ProcessState::Running => {
                if let (Some(pid), Some(started_at)) = (status.pid, status.started_at) {
                    Some(Self {
                        state_type: "running".to_string(),
                        pid: Some(pid),
                        started_at: Some(started_at.to_rfc3339()),
                        stopped_at: None,
                        exit_code: None,
                        error: None,
                    })
                } else {
                    None
                }
            }
            ProcessState::Stopped => status.stopped_at.map(|stopped_at| Self {
                state_type: "stopped".to_string(),
                pid: None,
                started_at: status.started_at.map(|t| t.to_rfc3339()),
                stopped_at: Some(stopped_at.to_rfc3339()),
                exit_code: status.exit_code,
                error: None,
            }),
            ProcessState::Failed => {
                if let (Some(error), Some(stopped_at)) = (&status.error, status.stopped_at) {
                    Some(Self {
                        state_type: "failed".to_string(),
                        pid: None,
                        started_at: status.started_at.map(|t| t.to_rfc3339()),
                        stopped_at: Some(stopped_at.to_rfc3339()),
                        exit_code: status.exit_code,
                        error: Some(error.clone()),
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Convert to ProcessStatus
    pub fn to_process_status(&self) -> ProcessStatus {
        let parse_datetime = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        };

        match self.state_type.as_str() {
            "running" => ProcessStatus {
                state: ProcessState::Running,
                pid: self.pid,
                exit_code: None,
                started_at: self.started_at.as_ref().and_then(|s| parse_datetime(s)),
                stopped_at: None,
                error: None,
            },
            "stopped" => ProcessStatus {
                state: ProcessState::Stopped,
                pid: None,
                exit_code: self.exit_code,
                started_at: self.started_at.as_ref().and_then(|s| parse_datetime(s)),
                stopped_at: self.stopped_at.as_ref().and_then(|s| parse_datetime(s)),
                error: None,
            },
            "failed" => ProcessStatus {
                state: ProcessState::Failed,
                pid: None,
                exit_code: self.exit_code,
                started_at: self.started_at.as_ref().and_then(|s| parse_datetime(s)),
                stopped_at: self.stopped_at.as_ref().and_then(|s| parse_datetime(s)),
                error: self.error.clone(),
            },
            _ => ProcessStatus::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdl_roundtrip() {
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
            status: ProcessStatus {
                state: ProcessState::Running,
                pid: Some(12345),
                exit_code: None,
                started_at: Some(Utc::now()),
                stopped_at: None,
                error: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["web".to_string()],
            auto_start_on_restore: true,
        };

        let snapshot = KdlSnapshot::from_processes(vec![process.clone()]);
        let kdl_string = snapshot.to_kdl_string().unwrap();

        // Check KDL content
        assert!(kdl_string.contains("process \"test-server\""));
        assert!(kdl_string.contains("name=\"Test Server\""));
        assert!(kdl_string.contains("command=\"python\""));

        // Parse back
        let parsed = KdlSnapshot::from_kdl_string(&kdl_string).unwrap();
        assert_eq!(parsed.processes.len(), 1);

        let parsed_process = &parsed.processes[0];
        assert_eq!(parsed_process.id, "test-server");
        assert_eq!(parsed_process.name, "Test Server");
        assert_eq!(parsed_process.command, "python");
        assert!(parsed_process.auto_start);
    }
}
