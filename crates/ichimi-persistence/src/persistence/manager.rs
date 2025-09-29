use crate::database::{Database, models::*, queries::*};
use crate::kdl_serde::KdlSnapshot;
use crate::types::{ClipboardItem, ProcessInfo, ProcessTemplate, Settings};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

// Type alias for simplified Result type
type Result<T> = std::result::Result<T, String>;

/// Persistence manager that uses SQLite for data storage and KDL for configuration
pub struct PersistenceManager {
    /// SQLite database connection
    database: Arc<Database>,

    /// KDL configuration file path
    config_path: PathBuf,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub async fn new(database_path: PathBuf, config_path: Option<PathBuf>) -> Result<Self> {
        // Initialize database
        let database = Database::new(&database_path)
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        let config_path = config_path.unwrap_or_else(Self::default_config_path);

        let manager = Self {
            database: Arc::new(database),
            config_path,
        };

        // Load configuration from KDL if exists
        if let Err(e) = manager.load_config().await {
            debug!("No existing KDL config to load: {}", e);
        }

        Ok(manager)
    }

    /// Get default configuration path
    fn default_config_path() -> PathBuf {
        // Use .ichimi directory in current working directory
        // for project-local configuration
        PathBuf::from(".ichimi").join("config.kdl")
    }

    /// Load configuration from KDL file
    async fn load_config(&self) -> Result<()> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let snapshot = KdlSnapshot::from_kdl_string(&content)
            .map_err(|e| format!("Failed to parse KDL config: {}", e))?;

        // Import processes from config
        for process in snapshot.processes {
            let db_process = process.to_process_info();
            self.save_process(&db_process).await?;
        }

        info!("Loaded configuration from {}", self.config_path.display());
        Ok(())
    }

    /// Save configuration to KDL file
    pub async fn save_config(&self) -> Result<()> {
        // Get all processes from database
        let processes = self.load_all_processes().await?;

        // Convert to KDL snapshot
        let snapshot = KdlSnapshot::from_processes(processes);
        let kdl_content = snapshot
            .to_kdl_string()
            .map_err(|e| format!("Failed to generate KDL: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Write to file
        tokio::fs::write(&self.config_path, kdl_content)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        info!("Saved configuration to {}", self.config_path.display());
        Ok(())
    }

    // Process management

    /// Save or update a process
    pub async fn save_process(&self, process: &ProcessInfo) -> Result<()> {
        ProcessQueries::upsert(self.database.pool(), process)
            .await
            .map_err(|e| format!("Failed to save process: {}", e))?;
        Ok(())
    }

    /// Update process information
    pub async fn update_process(&self, process: &ProcessInfo) -> Result<()> {
        self.save_process(process).await
    }

    /// Delete a process
    pub async fn delete_process(&self, process_id: &str) -> Result<()> {
        ProcessQueries::delete(self.database.pool(), process_id)
            .await
            .map_err(|e| format!("Failed to delete process: {}", e))?;
        Ok(())
    }

    /// Load all processes
    pub async fn load_all_processes(&self) -> Result<Vec<ProcessInfo>> {
        let records = ProcessQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to load processes: {}", e))?;

        let processes = records
            .into_iter()
            .map(|r| Self::record_to_process_info(r))
            .collect::<Result<Vec<_>>>()?;

        Ok(processes)
    }

    /// Get a specific process
    pub async fn get_process(&self, process_id: &str) -> Result<Option<ProcessInfo>> {
        let record = ProcessQueries::get(self.database.pool(), process_id)
            .await
            .map_err(|e| format!("Failed to get process: {}", e))?;

        match record {
            Some(r) => Ok(Some(Self::record_to_process_info(r)?)),
            None => Ok(None),
        }
    }

    // Template management

    /// Save a template
    pub async fn save_template(&self, template: &ProcessTemplate) -> Result<()> {
        ProcessTemplateQueries::upsert(self.database.pool(), template)
            .await
            .map_err(|e| format!("Failed to save template: {}", e))?;
        Ok(())
    }

    /// Get a template
    pub async fn get_template(&self, template_id: &str) -> Result<Option<ProcessTemplate>> {
        let record = ProcessTemplateQueries::get(self.database.pool(), template_id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;

        match record {
            Some(r) => Ok(Some(Self::record_to_template(r)?)),
            None => Ok(None),
        }
    }

    /// List all templates
    pub async fn list_templates(&self) -> Result<Vec<ProcessTemplate>> {
        let records = ProcessTemplateQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to list templates: {}", e))?;

        let templates = records
            .into_iter()
            .map(|r| Self::record_to_template(r))
            .collect::<Result<Vec<_>>>()?;

        Ok(templates)
    }

    /// Delete a template
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        ProcessTemplateQueries::delete(self.database.pool(), template_id)
            .await
            .map_err(|e| format!("Failed to delete template: {}", e))?;
        Ok(())
    }

    // Clipboard management

    /// Add to clipboard (legacy compatibility)
    pub async fn add_to_clipboard(
        &self,
        content: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<ClipboardItem> {
        let filename = metadata
            .as_ref()
            .and_then(|m| m.get("filename"))
            .and_then(|f| f.as_str())
            .map(|s| s.to_string());
        let item = ClipboardItem::new(content.clone(), filename, Some("text".to_string()));
        let key = format!(
            "clipboard_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );

        let metadata_str = metadata.map(|m| m.to_string());
        ClipboardQueries::store(
            self.database.pool(),
            &key,
            &content,
            "text",
            metadata_str,
            None,
        )
        .await
        .map_err(|e| format!("Failed to add to clipboard: {}", e))?;

        Ok(item)
    }

    /// Get clipboard history
    pub async fn get_clipboard_history(&self, limit: usize) -> Result<Vec<ClipboardItem>> {
        let records = ClipboardQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to get clipboard history: {}", e))?;

        let items = records
            .into_iter()
            .take(limit)
            .map(|r| {
                // Convert metadata back to filename if present
                let filename = r
                    .metadata
                    .and_then(|m| serde_json::from_str::<serde_json::Value>(&m).ok())
                    .and_then(|v| {
                        v.get("filename")
                            .and_then(|f| f.as_str())
                            .map(|s| s.to_string())
                    });
                ClipboardItem::new(r.content, filename, Some(r.content_type))
            })
            .collect();

        Ok(items)
    }

    /// Clear clipboard
    pub async fn clear_clipboard(&self) -> Result<()> {
        // Get all clipboard entries and delete them
        let records = ClipboardQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to list clipboard entries: {}", e))?;

        for record in records {
            ClipboardQueries::delete(self.database.pool(), &record.key)
                .await
                .map_err(|e| format!("Failed to delete clipboard entry: {}", e))?;
        }

        Ok(())
    }

    /// Get latest clipboard item
    pub async fn get_latest_clipboard_item(&self) -> Result<Option<ClipboardItem>> {
        let records = ClipboardQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to get clipboard: {}", e))?;

        Ok(records.first().map(|r| {
            let filename = r
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
                .and_then(|v| {
                    v.get("filename")
                        .and_then(|f| f.as_str())
                        .map(|s| s.to_string())
                });
            ClipboardItem::new(r.content.clone(), filename, Some(r.content_type.clone()))
        }))
    }

    /// Set clipboard text
    pub async fn set_clipboard_text(&self, text: String) -> Result<ClipboardItem> {
        self.add_to_clipboard(text, None).await
    }

    /// Save clipboard item
    pub async fn save_clipboard_item(&self, item: &ClipboardItem) -> Result<()> {
        let key = item.id.clone().unwrap_or_else(|| {
            format!(
                "clipboard_{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
            )
        });

        let metadata = item
            .filename
            .as_ref()
            .map(|f| serde_json::json!({ "filename": f }).to_string());

        ClipboardQueries::store(
            self.database.pool(),
            &key,
            &item.content,
            item.content_type.as_deref().unwrap_or("text"),
            metadata,
            None,
        )
        .await
        .map_err(|e| format!("Failed to save clipboard item: {}", e))?;

        Ok(())
    }

    /// Get clipboard text
    pub async fn get_clipboard_text(&self) -> Result<Option<String>> {
        let item = self.get_latest_clipboard_item().await?;
        Ok(item.map(|i| i.content))
    }

    // Settings management

    /// Get settings
    pub async fn get_settings(&self) -> Result<Settings> {
        let records = SettingsQueries::list(self.database.pool())
            .await
            .map_err(|e| format!("Failed to get settings: {}", e))?;

        // Convert settings records to Settings struct
        let mut settings = Settings::default();
        for record in records {
            match record.key.as_str() {
                "theme" => {
                    settings.theme = record.value;
                }
                "auto_save_interval" => {
                    settings.auto_save_interval = record.value.parse().ok();
                }
                "max_log_lines" => {
                    settings.max_log_lines = record.value.parse().ok();
                }
                "enable_auto_restart" => {
                    settings.enable_auto_restart = record.value.parse().unwrap_or(false);
                }
                "default_shell" => {
                    settings.default_shell = Some(record.value);
                }
                _ => {
                    // Store as environment variable
                    if record.key.starts_with("env_") {
                        let env_key = record.key.strip_prefix("env_").unwrap_or(&record.key);
                        settings
                            .env_variables
                            .insert(env_key.to_string(), record.value);
                    }
                }
            }
        }

        Ok(settings)
    }

    /// Update settings
    pub async fn update_settings(&self, settings: Settings) -> Result<()> {
        // Save each setting to database
        SettingsQueries::set(self.database.pool(), "theme", &settings.theme)
            .await
            .map_err(|e| format!("Failed to update settings: {}", e))?;

        if let Some(interval) = settings.auto_save_interval {
            SettingsQueries::set(
                self.database.pool(),
                "auto_save_interval",
                &interval.to_string(),
            )
            .await
            .map_err(|e| format!("Failed to update settings: {}", e))?;
        }

        if let Some(max_lines) = settings.max_log_lines {
            SettingsQueries::set(
                self.database.pool(),
                "max_log_lines",
                &max_lines.to_string(),
            )
            .await
            .map_err(|e| format!("Failed to update settings: {}", e))?;
        }

        SettingsQueries::set(
            self.database.pool(),
            "enable_auto_restart",
            &settings.enable_auto_restart.to_string(),
        )
        .await
        .map_err(|e| format!("Failed to update settings: {}", e))?;

        if let Some(shell) = settings.default_shell {
            SettingsQueries::set(self.database.pool(), "default_shell", &shell)
                .await
                .map_err(|e| format!("Failed to update settings: {}", e))?;
        }

        // Save environment variables
        for (key, value) in settings.env_variables {
            SettingsQueries::set(self.database.pool(), &format!("env_{}", key), &value)
                .await
                .map_err(|e| format!("Failed to update settings: {}", e))?;
        }

        Ok(())
    }

    // Database event recording

    /// Record process start in database
    pub async fn record_process_start(&self, process: &ProcessInfo) -> Result<()> {
        let args_json = serde_json::to_string(&process.args)
            .map_err(|e| format!("Failed to serialize args: {}", e))?;
        let env_json = serde_json::to_string(&process.env)
            .map_err(|e| format!("Failed to serialize env: {}", e))?;

        ProcessHistoryQueries::record_start(
            self.database.pool(),
            &process.process_id,
            &process.name,
            &process.command,
            Some(args_json),
            Some(env_json),
            process.cwd.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to record process start: {}", e))?;

        Ok(())
    }

    /// Record process stop in database
    pub async fn record_process_stop(
        &self,
        process_id: &str,
        exit_code: Option<i32>,
        error: Option<&str>,
    ) -> Result<()> {
        ProcessHistoryQueries::record_stop(self.database.pool(), process_id, exit_code, error)
            .await
            .map_err(|e| format!("Failed to record process stop: {}", e))?;

        Ok(())
    }

    /// Record system event
    pub async fn record_system_event(
        &self,
        event_type: &str,
        description: &str,
        details: Option<String>,
        severity: &str,
    ) -> Result<()> {
        SystemEventQueries::record(
            self.database.pool(),
            event_type,
            description,
            details,
            severity,
        )
        .await
        .map_err(|e| format!("Failed to record system event: {}", e))?;

        Ok(())
    }

    /// Get process history from database
    pub async fn get_process_history(
        &self,
        process_id: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ProcessHistoryRecord>> {
        ProcessHistoryQueries::get_history(self.database.pool(), process_id, limit)
            .await
            .map_err(|e| format!("Failed to get process history: {}", e))
    }

    // Conversion helpers

    /// Convert database record to ProcessInfo
    fn record_to_process_info(record: ProcessRecord) -> Result<ProcessInfo> {
        let args: Vec<String> = record
            .args
            .map(|a| serde_json::from_str(&a))
            .transpose()
            .map_err(|e| format!("Failed to parse args: {}", e))?
            .unwrap_or_default();

        let env: std::collections::HashMap<String, String> = record
            .env
            .map(|e| serde_json::from_str(&e))
            .transpose()
            .map_err(|e| format!("Failed to parse env: {}", e))?
            .unwrap_or_default();

        let tags: Vec<String> = record
            .tags
            .map(|t| serde_json::from_str(&t))
            .transpose()
            .map_err(|e| format!("Failed to parse tags: {}", e))?
            .unwrap_or_default();

        let state = match record.state.as_str() {
            "running" => crate::types::ProcessState::Running,
            "stopped" => crate::types::ProcessState::Stopped,
            "failed" => crate::types::ProcessState::Failed,
            _ => crate::types::ProcessState::NotStarted,
        };

        Ok(ProcessInfo {
            id: Some(record.id.to_string()),
            process_id: record.process_id,
            name: record.name,
            command: record.command,
            args,
            env,
            cwd: record.cwd,
            status: crate::types::ProcessStatus {
                state,
                pid: record.pid.map(|p| p as u32),
                exit_code: record.exit_code,
                started_at: record.started_at,
                stopped_at: record.stopped_at,
                error: record.error,
            },
            created_at: record.created_at,
            updated_at: record.updated_at,
            tags,
            auto_start_on_restore: record.auto_start_on_restore,
        })
    }

    /// Convert database record to ProcessTemplate
    fn record_to_template(record: ProcessTemplateRecord) -> Result<ProcessTemplate> {
        let args: Vec<String> = record
            .args
            .map(|a| serde_json::from_str(&a))
            .transpose()
            .map_err(|e| format!("Failed to parse args: {}", e))?
            .unwrap_or_default();

        let env: std::collections::HashMap<String, String> = record
            .env
            .map(|e| serde_json::from_str(&e))
            .transpose()
            .map_err(|e| format!("Failed to parse env: {}", e))?
            .unwrap_or_default();

        let variables: Vec<crate::types::TemplateVariable> = record
            .variables
            .map(|v| serde_json::from_str(&v))
            .transpose()
            .map_err(|e| format!("Failed to parse variables: {}", e))?
            .unwrap_or_default();

        let tags: Vec<String> = record
            .tags
            .map(|t| serde_json::from_str(&t))
            .transpose()
            .map_err(|e| format!("Failed to parse tags: {}", e))?
            .unwrap_or_default();

        Ok(ProcessTemplate {
            id: Some(record.id.to_string()),
            template_id: record.template_id,
            name: record.name,
            description: record.description,
            category: record.category,
            command: record.command,
            args,
            env,
            default_cwd: record.default_cwd,
            default_auto_start: record.default_auto_start,
            variables,
            created_at: record.created_at,
            updated_at: record.updated_at,
            tags,
        })
    }

    // Legacy compatibility (these methods now do nothing as we don't use YAML)

    /// Export to YAML (deprecated - does nothing)
    pub async fn export_to_yaml(&self, _path: &Path) -> Result<()> {
        warn!("YAML export is deprecated");
        Ok(())
    }

    /// Import from YAML (deprecated - does nothing)
    pub async fn import_from_yaml(&self, _path: &Path) -> Result<()> {
        warn!("YAML import is deprecated");
        Ok(())
    }

    /// Export snapshot (deprecated - saves config instead)
    pub async fn export_snapshot(
        &self,
        _path: Option<&Path>,
        _filter_auto_start: bool,
    ) -> Result<()> {
        self.save_config().await
    }

    /// Import snapshot (deprecated - loads config instead)
    pub async fn import_snapshot(&self, _path: Option<&Path>) -> Result<()> {
        self.load_config().await
    }

    /// Create auto-start snapshot (deprecated - does nothing)
    pub async fn create_auto_start_snapshot(&self) -> Result<()> {
        Ok(())
    }

    /// Restore snapshot (deprecated - loads config instead)
    pub async fn restore_snapshot(&self, _file_path: Option<&str>) -> Result<()> {
        self.load_config().await
    }

    /// Restore YAML snapshot (deprecated - does nothing)
    pub async fn restore_yaml_snapshot(&self, _path: &Path) -> Result<()> {
        warn!("YAML restore is deprecated");
        Ok(())
    }

    /// Export to file (deprecated - saves config instead)
    pub async fn export_to_file(&self, _path: &Path, _filter_auto_start: bool) -> Result<()> {
        self.save_config().await
    }

    /// Import from file (deprecated - loads config instead)
    pub async fn import_from_file(&self, _path: &Path) -> Result<()> {
        self.load_config().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence_manager() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config_path = temp_dir.path().join("config.kdl");

        let manager = PersistenceManager::new(db_path, Some(config_path))
            .await
            .unwrap();

        // Test process operations
        let mut process = ProcessInfo {
            id: None,
            process_id: "test-process".to_string(),
            name: "Test Process".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: std::collections::HashMap::new(),
            cwd: None,
            status: crate::types::ProcessStatus::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec!["test".to_string()],
            auto_start_on_restore: false,
        };

        // Save process
        manager.save_process(&process).await.unwrap();

        // Get process
        let loaded = manager.get_process("test-process").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "Test Process");

        // Update process
        process.name = "Updated Process".to_string();
        manager.update_process(&process).await.unwrap();

        // List processes
        let processes = manager.load_all_processes().await.unwrap();
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].name, "Updated Process");

        // Delete process
        manager.delete_process("test-process").await.unwrap();
        let deleted = manager.get_process("test-process").await.unwrap();
        assert!(deleted.is_none());
    }
}
