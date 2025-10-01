use crate::types::{ClipboardItem, ProcessInfo, ProcessTemplate, Settings};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Type alias for simplified Result type
type Result<T> = std::result::Result<T, String>;

/// Persistence manager for in-memory storage with YAML snapshot support
#[derive(Clone)]
pub struct PersistenceManager {
    #[allow(dead_code)]
    snapshot_path: PathBuf,
    #[allow(dead_code)]
    snapshot_lock: Arc<tokio::sync::RwLock<()>>,
    processes: Arc<tokio::sync::RwLock<HashMap<String, ProcessInfo>>>,
    templates: Arc<tokio::sync::RwLock<HashMap<String, ProcessTemplate>>>,
    clipboard: Arc<tokio::sync::RwLock<Vec<ClipboardItem>>>,
    settings: Arc<tokio::sync::RwLock<Settings>>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub async fn new() -> Result<Self> {
        let snapshot_path = Self::default_snapshot_path();
        let snapshot_lock = Arc::new(tokio::sync::RwLock::new(()));
        let processes = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let templates = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let clipboard = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        let settings = Arc::new(tokio::sync::RwLock::new(Settings::default()));

        Ok(Self {
            snapshot_path,
            snapshot_lock,
            processes,
            templates,
            clipboard,
            settings,
        })
    }

    /// Get default snapshot path
    fn default_snapshot_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".ichimi").join("snapshot.yaml")
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
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>> {
        let processes = self.processes.read().await;
        Ok(processes.clone())
    }

    /// Export processes to YAML snapshot
    pub async fn export_snapshot(
        &self,
        file_path: Option<&str>,
        only_auto_start: bool,
    ) -> Result<String> {
        let path = match file_path {
            Some(p) => PathBuf::from(p),
            None => self.snapshot_path.clone(),
        };

        let processes = self.load_all_processes().await?;
        let mut process_list: Vec<ProcessInfo> = processes.into_values().collect();

        if only_auto_start {
            process_list.retain(|p| p.auto_start_on_restore);
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        let yaml = serde_yaml::to_string(&process_list)
            .map_err(|e| format!("Failed to serialize to YAML: {e}"))?;

        tokio::fs::write(&path, yaml)
            .await
            .map_err(|e| format!("Failed to write snapshot: {e}"))?;

        tracing::info!(
            "Exported {} processes to YAML snapshot (auto_start_only: {})",
            process_list.len(),
            only_auto_start
        );

        Ok(path.to_string_lossy().to_string())
    }

    /// Import processes from YAML snapshot
    pub async fn import_snapshot(
        &self,
        file_path: Option<&str>,
    ) -> Result<HashMap<String, ProcessInfo>> {
        let path = match file_path {
            Some(p) => Path::new(p),
            None => &self.snapshot_path,
        };

        if !path.exists() {
            return Err(format!("Snapshot file not found: {}", path.display()));
        }

        let yaml = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read snapshot: {e}"))?;

        let process_list: Vec<ProcessInfo> =
            serde_yaml::from_str(&yaml).map_err(|e| format!("Failed to deserialize YAML: {e}"))?;

        let mut imported = HashMap::new();
        let mut processes = self.processes.write().await;

        for process_info in process_list {
            let process_id = process_info.process_id.clone();
            processes.insert(process_id.clone(), process_info.clone());
            imported.insert(process_id, process_info);
        }

        tracing::info!("Imported {} processes from YAML snapshot", imported.len());

        Ok(imported)
    }

    /// Create an auto-start snapshot
    pub async fn create_auto_start_snapshot(&self, file_path: Option<&str>) -> Result<String> {
        self.export_snapshot(file_path, true).await
    }

    /// Restore from snapshot
    pub async fn restore_snapshot(
        &self,
        file_path: Option<&str>,
    ) -> Result<HashMap<String, ProcessInfo>> {
        self.import_snapshot(file_path).await
    }

    // YAML snapshot methods

    /// Export to YAML
    pub async fn export_to_yaml(&self, file_path: &str, only_auto_start: bool) -> Result<()> {
        self.export_snapshot(Some(file_path), only_auto_start)
            .await?;
        Ok(())
    }

    /// Import from YAML
    pub async fn import_from_yaml(&self, file_path: &str) -> Result<HashMap<String, ProcessInfo>> {
        self.import_snapshot(Some(file_path)).await
    }

    /// Restore YAML snapshot
    pub async fn restore_yaml_snapshot(
        &self,
        file_path: Option<&str>,
    ) -> Result<HashMap<String, ProcessInfo>> {
        self.restore_snapshot(file_path).await
    }

    // JSON export/import for REST API

    /// Export to JSON file
    pub async fn export_to_file(&self, file_path: &str) -> Result<()> {
        let processes = self.load_all_processes().await?;
        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| format!("Failed to serialize processes: {e}"))?;

        std::fs::write(file_path, json).map_err(|e| format!("Failed to write export file: {e}"))?;

        tracing::info!("Exported {} processes to {}", processes.len(), file_path);
        Ok(())
    }

    /// Import from JSON file
    pub async fn import_from_file(&self, file_path: &str) -> Result<()> {
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

    /// Save a template
    pub async fn save_template(&self, template: &ProcessTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.template_id.clone(), template.clone());
        tracing::info!("Saved template {}", template.template_id);
        Ok(())
    }

    /// Get a template
    pub async fn get_template(&self, template_id: &str) -> Result<Option<ProcessTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(template_id).cloned())
    }

    /// List all templates
    pub async fn list_templates(&self) -> Result<Vec<ProcessTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.values().cloned().collect())
    }

    /// Delete a template
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.remove(template_id);
        tracing::info!("Deleted template {}", template_id);
        Ok(())
    }

    // Clipboard management

    /// Add to clipboard
    pub async fn add_to_clipboard(&self, text: String) -> Result<()> {
        let mut clipboard = self.clipboard.write().await;
        clipboard.push(ClipboardItem::new(text, None, None));

        // Keep only last 100 items
        if clipboard.len() > 100 {
            let drain_count = clipboard.len() - 100;
            clipboard.drain(0..drain_count);
        }

        Ok(())
    }

    /// Get clipboard history
    pub async fn get_clipboard_history(&self, limit: Option<usize>) -> Result<Vec<ClipboardItem>> {
        let clipboard = self.clipboard.read().await;
        let limit = limit.unwrap_or(10).min(clipboard.len());
        Ok(clipboard.iter().rev().take(limit).cloned().collect())
    }

    /// Clear clipboard
    pub async fn clear_clipboard(&self) -> Result<()> {
        let mut clipboard = self.clipboard.write().await;
        clipboard.clear();
        Ok(())
    }

    /// Get latest clipboard item
    pub async fn get_latest_clipboard_item(&self) -> Result<Option<ClipboardItem>> {
        let clipboard = self.clipboard.read().await;
        Ok(clipboard.last().cloned())
    }

    /// Set clipboard text (for compatibility)
    pub async fn set_clipboard_text(&self, text: String) -> Result<ClipboardItem> {
        self.add_to_clipboard(text)
            .await
            .map_err(|e| format!("Failed to set clipboard: {e}"))?;

        // Return the latest clipboard item (should be the one we just added)
        self.get_latest_clipboard_item()
            .await?
            .ok_or_else(|| "Failed to retrieve clipboard item after adding".to_string())
    }

    /// Get clipboard text (for compatibility)
    pub async fn get_clipboard_text(&self) -> Result<Option<String>> {
        let item = self
            .get_latest_clipboard_item()
            .await
            .map_err(|e| format!("Failed to get clipboard: {e}"))?;
        Ok(item.map(|i| i.content))
    }

    /// Save/update clipboard item
    pub async fn save_clipboard_item(&self, item: &ClipboardItem) -> Result<()> {
        let mut clipboard = self.clipboard.write().await;

        // Find and update existing item by ID
        if let Some(existing) = clipboard
            .iter_mut()
            .find(|i| i.clipboard_id == item.clipboard_id)
        {
            *existing = item.clone();
        } else {
            // If not found, add as new item
            clipboard.push(item.clone());
        }

        Ok(())
    }

    /// Search clipboard items
    pub async fn search_clipboard_items(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ClipboardItem>> {
        let clipboard = self.clipboard.read().await;
        let query_lower = query.to_lowercase();

        let results: Vec<ClipboardItem> = clipboard
            .iter()
            .filter(|item| {
                item.content.to_lowercase().contains(&query_lower)
                    || item
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
                    || item
                        .filename
                        .as_ref()
                        .map(|f| f.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .rev()
            .take(limit)
            .cloned()
            .collect();

        Ok(results)
    }

    // Settings management

    /// Get settings
    pub async fn get_settings(&self) -> Result<Settings> {
        let settings = self.settings.read().await;
        Ok(settings.clone())
    }

    /// Update settings
    pub async fn update_settings(&self, settings: Settings) -> Result<()> {
        let mut current = self.settings.write().await;
        *current = settings;
        Ok(())
    }
}
