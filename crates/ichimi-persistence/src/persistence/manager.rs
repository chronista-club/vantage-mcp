use crate::types::{ClipboardItem, ProcessInfo, ProcessTemplate, Settings};
use crate::yaml::{ProcessSnapshot, Snapshot};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Persistence manager for file-based storage
#[derive(Clone)]
pub struct PersistenceManager {
    #[allow(dead_code)]
    snapshot_lock: Arc<tokio::sync::RwLock<()>>,
    // In-memory storage for now, will be replaced with file-based persistence
    processes: Arc<tokio::sync::RwLock<HashMap<String, ProcessInfo>>>,
    templates: Arc<tokio::sync::RwLock<HashMap<String, ProcessTemplate>>>,
    clipboard: Arc<tokio::sync::RwLock<Vec<ClipboardItem>>>,
    settings: Arc<tokio::sync::RwLock<Settings>>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub async fn new() -> Result<Self> {
        let snapshot_lock = Arc::new(tokio::sync::RwLock::new(()));
        let processes = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let templates = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let clipboard = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        let settings = Arc::new(tokio::sync::RwLock::new(Settings::default()));

        Ok(Self {
            snapshot_lock,
            processes,
            templates,
            clipboard,
            settings,
        })
    }

    /// Save or update a process
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<()> {
        let mut processes = self.processes.write().await;
        processes.insert(process_info.process_id.clone(), process_info.clone());
        tracing::info!("Saved process {}", process_info.process_id);
        Ok(())
    }

    /// Update a process (alias for save_process)
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<()> {
        self.save_process(process_info).await
    }

    /// Delete a process
    pub async fn delete_process(&self, process_id: &str) -> Result<()> {
        let mut processes = self.processes.write().await;
        processes.remove(process_id);
        tracing::info!("Deleted process {}", process_id);
        Ok(())
    }

    /// Load all processes
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let processes = self.processes.read().await;
        Ok(processes.clone())
    }

    /// Export processes to YAML format
    pub async fn export_to_yaml(
        &self,
        file_path: &str,
        only_auto_start: bool,
    ) -> Result<(), String> {
        let processes = self.load_all_processes().await?;

        let snapshots: Vec<ProcessSnapshot> =
            processes.values().map(ProcessSnapshot::from).collect();

        let snapshot = if only_auto_start {
            Snapshot::new_auto_start_only(snapshots)
        } else {
            Snapshot::new(snapshots)
        };

        snapshot
            .save_to_file(Path::new(file_path))
            .await
            .map_err(|e| format!("Failed to save YAML snapshot: {e}"))?;

        tracing::info!(
            "Exported {} processes to YAML (auto_start_only: {})",
            snapshot.processes.len(),
            only_auto_start
        );

        Ok(())
    }

    /// Import processes from YAML format
    pub async fn import_from_yaml(
        &self,
        file_path: &str,
    ) -> Result<HashMap<String, ProcessInfo>, String> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(format!("YAML file not found: {file_path}"));
        }

        let snapshot = Snapshot::load_from_file(path)
            .await
            .map_err(|e| format!("Failed to load YAML snapshot: {e}"))?;

        if !snapshot.is_compatible() {
            return Err(format!(
                "Incompatible snapshot version: {} (expected: 1.0)",
                snapshot.version
            ));
        }

        let mut imported_processes = HashMap::new();
        let mut processes = self.processes.write().await;

        for process_snapshot in snapshot.processes {
            let process_info = process_snapshot.to_process_info();
            let process_id = process_info.process_id.clone();
            processes.insert(process_id.clone(), process_info.clone());
            imported_processes.insert(process_id, process_info);
        }

        tracing::info!(
            "Imported {} processes from YAML (created at: {})",
            imported_processes.len(),
            snapshot.timestamp
        );

        Ok(imported_processes)
    }

    /// Create a YAML snapshot with only auto-start processes
    pub async fn create_auto_start_snapshot(
        &self,
        file_path: Option<&str>,
    ) -> Result<String, String> {
        let path = match file_path {
            Some(p) => p.to_string(),
            None => {
                let snapshot_dir = std::env::var("HOME")
                    .map(|home| format!("{home}/.ichimi"))
                    .unwrap_or_else(|_| ".ichimi".to_string());
                format!("{snapshot_dir}/snapshot.yaml")
            }
        };

        self.export_to_yaml(&path, true).await?;
        Ok(path)
    }

    /// Restore processes from YAML snapshot
    pub async fn restore_yaml_snapshot(
        &self,
        file_path: Option<&str>,
    ) -> Result<HashMap<String, ProcessInfo>, String> {
        let path = match file_path {
            Some(p) => p.to_string(),
            None => {
                let snapshot_dir = std::env::var("HOME")
                    .map(|home| format!("{home}/.ichimi"))
                    .unwrap_or_else(|_| ".ichimi".to_string());
                format!("{snapshot_dir}/snapshot.yaml")
            }
        };

        if !std::path::Path::new(&path).exists() {
            return Err(format!("YAML snapshot not found: {path}"));
        }

        self.import_from_yaml(&path).await
    }

    /// Export to JSON file (compatibility)
    pub async fn export_to_file(&self, file_path: &str) -> Result<(), String> {
        let processes = self.load_all_processes().await?;
        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| format!("Failed to serialize processes: {e}"))?;

        std::fs::write(file_path, json).map_err(|e| format!("Failed to write export file: {e}"))?;

        tracing::info!("Exported {} processes to {}", processes.len(), file_path);
        Ok(())
    }

    /// Import from JSON file (compatibility)
    pub async fn import_from_file(&self, file_path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read import file: {e}"))?;

        let imported: HashMap<String, ProcessInfo> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to deserialize processes: {e}"))?;

        let mut processes = self.processes.write().await;
        for (id, info) in imported.iter() {
            processes.insert(id.clone(), info.clone());
        }

        tracing::info!("Imported {} processes from {}", imported.len(), file_path);
        Ok(())
    }

    // Template management
    pub async fn save_template(&self, template: &ProcessTemplate) -> Result<(), String> {
        let mut templates = self.templates.write().await;
        templates.insert(template.template_id.clone(), template.clone());
        tracing::info!("Saved template {}", template.template_id);
        Ok(())
    }

    pub async fn get_template(&self, template_id: &str) -> Result<ProcessTemplate, String> {
        let templates = self.templates.read().await;
        templates
            .get(template_id)
            .cloned()
            .ok_or_else(|| format!("Template {template_id} not found"))
    }

    pub async fn list_templates(&self) -> Result<Vec<ProcessTemplate>, String> {
        let templates = self.templates.read().await;
        Ok(templates.values().cloned().collect())
    }

    pub async fn delete_template(&self, template_id: &str) -> Result<(), String> {
        let mut templates = self.templates.write().await;
        templates.remove(template_id);
        tracing::info!("Deleted template {}", template_id);
        Ok(())
    }

    // Clipboard management
    pub async fn save_clipboard_item(&self, item: &ClipboardItem) -> Result<(), String> {
        let mut clipboard = self.clipboard.write().await;
        clipboard.push(item.clone());

        // Keep only the last 100 items
        let len = clipboard.len();
        if len > 100 {
            clipboard.drain(0..len - 100);
        }

        tracing::debug!("Saved clipboard item");
        Ok(())
    }

    pub async fn get_clipboard_history(&self, limit: usize) -> Result<Vec<ClipboardItem>, String> {
        let clipboard = self.clipboard.read().await;
        let items: Vec<ClipboardItem> = clipboard.iter().rev().take(limit).cloned().collect();
        Ok(items)
    }

    pub async fn clear_clipboard(&self) -> Result<(), String> {
        let mut clipboard = self.clipboard.write().await;
        clipboard.clear();
        tracing::info!("Cleared clipboard history");
        Ok(())
    }

    /// Get the latest clipboard item
    pub async fn get_latest_clipboard_item(&self) -> Result<ClipboardItem, String> {
        let clipboard = self.clipboard.read().await;
        clipboard
            .last()
            .cloned()
            .ok_or_else(|| "No clipboard items available".to_string())
    }

    /// Set clipboard with text content
    pub async fn set_clipboard_text(&self, content: String) -> Result<ClipboardItem, String> {
        let item = ClipboardItem {
            id: None,
            clipboard_id: crate::types::generate_id(),
            content,
            filename: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            content_type: Some("text/plain".to_string()),
            tags: Vec::new(),
        };
        self.save_clipboard_item(&item).await?;
        Ok(item)
    }

    /// Search clipboard items by content
    pub async fn search_clipboard_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ClipboardItem>, String> {
        let clipboard = self.clipboard.read().await;
        let query_lower = query.to_lowercase();
        let items: Vec<ClipboardItem> = clipboard
            .iter()
            .filter(|item| item.content.to_lowercase().contains(&query_lower))
            .rev()
            .take(limit)
            .cloned()
            .collect();
        Ok(items)
    }

    // Settings management
    pub async fn get_settings(&self) -> Result<Settings, String> {
        let settings = self.settings.read().await;
        Ok(settings.clone())
    }

    pub async fn update_settings(&self, new_settings: Settings) -> Result<(), String> {
        let mut settings = self.settings.write().await;
        *settings = new_settings;
        tracing::info!("Updated settings");
        Ok(())
    }

    /// Query processes with simple filters
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let processes = self.processes.read().await;

        if filter.is_empty() {
            Ok(processes.values().cloned().collect())
        } else {
            let filtered: Vec<ProcessInfo> = processes
                .values()
                .filter(|p| p.command.contains(filter))
                .cloned()
                .collect();
            Ok(filtered)
        }
    }

    /// Get process statistics
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let processes = self.processes.read().await;

        let mut running_count = 0;
        let mut stopped_count = 0;
        let mut failed_count = 0;
        let mut not_started_count = 0;

        for process in processes.values() {
            match process.status.state {
                crate::types::ProcessState::Running => running_count += 1,
                crate::types::ProcessState::Stopped => stopped_count += 1,
                crate::types::ProcessState::Failed => failed_count += 1,
                crate::types::ProcessState::NotStarted => not_started_count += 1,
            }
        }

        Ok(serde_json::json!({
            "total_processes": processes.len(),
            "running_count": running_count,
            "stopped_count": stopped_count,
            "failed_count": failed_count,
            "not_started_count": not_started_count,
        }))
    }

    /// Search processes by command or args
    /// Save settings
    pub async fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let mut settings_guard = self.settings.write().await;
        *settings_guard = settings.clone();
        tracing::info!("Saved settings");
        Ok(())
    }

    /// Load all templates (alias for list_templates)
    pub async fn load_all_templates(&self) -> Result<Vec<ProcessTemplate>, String> {
        self.list_templates().await
    }

    /// Search templates by name or description
    pub async fn search_templates(&self, query: &str) -> Result<Vec<ProcessTemplate>, String> {
        let templates = self.templates.read().await;
        let query_lower = query.to_lowercase();
        let filtered: Vec<ProcessTemplate> = templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        Ok(filtered)
    }

    pub async fn search_processes(&self, search_term: &str) -> Result<Vec<ProcessInfo>, String> {
        let processes = self.processes.read().await;
        let search_lower = search_term.to_lowercase();

        let filtered: Vec<ProcessInfo> = processes
            .values()
            .filter(|p| {
                p.command.to_lowercase().contains(&search_lower)
                    || p.args
                        .iter()
                        .any(|arg| arg.to_lowercase().contains(&search_lower))
            })
            .cloned()
            .collect();

        Ok(filtered)
    }
}
