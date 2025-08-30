use super::kdl_persistence::KdlPersistence;
use super::kdl_schema::ProcessConfig;
use crate::db::Database;
use crate::process::types::{ProcessInfo, ProcessState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessInfoRecord {
    id: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<String>,
    auto_start: bool,
}

impl From<&ProcessInfo> for ProcessInfoRecord {
    fn from(info: &ProcessInfo) -> Self {
        Self {
            id: info.id.clone(),
            command: info.command.clone(),
            args: info.args.clone(),
            env: info.env.clone(),
            cwd: info.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
            auto_start: info.auto_start,
        }
    }
}

impl ProcessInfoRecord {
    fn to_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: self.id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.as_ref().map(PathBuf::from),
            state: ProcessState::NotStarted,
            auto_start: self.auto_start,
        }
    }
}

/// Persistence manager using SurrealDB for storage
pub struct PersistenceManager {
    database: Arc<Database>,
}

impl PersistenceManager {
    /// Create a new persistence manager with SurrealDB storage
    pub async fn new() -> Result<Self> {
        let database = Arc::new(Database::new().await?);
        Ok(Self { database })
    }

    /// Create with existing database instance
    pub fn with_database(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Save or update a process in SurrealDB
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<()> {
        tracing::info!("Attempting to save process {} to SurrealDB", process_info.id);
        let record = ProcessInfoRecord::from(process_info);
        let client = self.database.client().await;

        // UPSERTを使用して、存在する場合は更新、しない場合は作成
        let query = r#"
            UPSERT process_info:[$id] CONTENT {
                id: $id,
                command: $command,
                args: $args,
                env: $env,
                cwd: $cwd,
                auto_start: $auto_start,
                updated_at: time::now()
            }
        "#;

        client
            .query(query)
            .bind(("id", record.id.clone()))
            .bind(("command", record.command.clone()))
            .bind(("args", record.args.clone()))
            .bind(("env", record.env.clone()))
            .bind(("cwd", record.cwd.clone()))
            .bind(("auto_start", record.auto_start))
            .await
            .context("Failed to save process to SurrealDB")?;

        tracing::debug!("Saved process {} to SurrealDB", process_info.id);
        Ok(())
    }

    /// Update a process (alias for save_process)
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<()> {
        self.save_process(process_info).await
    }

    /// Delete a process from SurrealDB
    pub async fn delete_process(&self, process_id: &str) -> Result<()> {
        let client = self.database.client().await;

        let query = "DELETE process_info:[$id]";
        client
            .query(query)
            .bind(("id", process_id.to_string()))
            .await
            .context("Failed to delete process from SurrealDB")?;

        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }

    /// Load all processes from SurrealDB
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let client = self.database.client().await;

        let query = "SELECT * FROM process_info";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query processes: {}", e))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse process records: {}", e))?;

        let mut result = HashMap::new();
        for record in records {
            let info = record.to_process_info();
            result.insert(info.id.clone(), info);
        }

        tracing::info!("Loaded {} processes from SurrealDB", result.len());
        Ok(result)
    }

    /// Query processes with simple filters
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let client = self.database.client().await;

        let (query, bind_filter) = if filter.is_empty() {
            ("SELECT * FROM process_info".to_string(), false)
        } else {
            (
                "SELECT * FROM process_info WHERE command CONTAINS $filter".to_string(),
                true,
            )
        };

        let mut query_builder = client.query(&query);
        if bind_filter {
            query_builder = query_builder.bind(("filter", filter.to_string()));
        }

        let mut response = query_builder
            .await
            .map_err(|e| format!("Failed to query processes: {}", e))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse process records: {}", e))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
    }

    /// Get process statistics from SurrealDB
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let client = self.database.client().await;

        let query = "SELECT count() as total FROM process_info GROUP ALL";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to get stats: {}", e))?;

        let stats: Option<serde_json::Value> = response
            .take(0)
            .map_err(|e| format!("Failed to parse stats: {}", e))?;

        Ok(stats.unwrap_or(serde_json::json!({
            "total_processes": 0,
            "running_count": 0,
            "stopped_count": 0,
            "failed_count": 0
        })))
    }

    /// Search processes by command or args
    pub async fn search_processes(&self, search_term: &str) -> Result<Vec<ProcessInfo>, String> {
        let client = self.database.client().await;

        let query = r#"
            SELECT * FROM process_info 
            WHERE command CONTAINS $term 
               OR array::any(args, |arg| string::lowercase(arg) CONTAINS $term_lower)
        "#;

        let mut response = client
            .query(query)
            .bind(("term", search_term.to_string()))
            .bind(("term_lower", search_term.to_lowercase()))
            .await
            .map_err(|e| format!("Failed to search processes: {}", e))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse search results: {}", e))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
    }

    /// Export processes to KDL file
    pub async fn export_to_kdl(&self, file_path: &str) -> Result<(), String> {
        let processes = self.load_all_processes().await?;

        // KDL形式でエクスポート
        let config_dir = PathBuf::from(".ichimi_export");
        let kdl_persistence = KdlPersistence::new(&config_dir);

        for (_, info) in processes.iter() {
            let process_config = ProcessConfig::from_process_info(info);
            kdl_persistence
                .add_or_update_process(process_config)
                .map_err(|e| format!("Failed to export process to KDL: {}", e))?;
        }

        // エクスポートディレクトリから指定パスにコピー
        let export_file = config_dir.join("processes.kdl");
        std::fs::copy(&export_file, file_path)
            .map_err(|e| format!("Failed to copy export file: {}", e))?;

        // 一時ディレクトリをクリーンアップ
        let _ = std::fs::remove_dir_all(&config_dir);

        tracing::info!("Exported {} processes to KDL file: {}", processes.len(), file_path);
        Ok(())
    }

    /// Import processes from KDL file
    pub async fn import_from_kdl(&self, file_path: &str) -> Result<(), String> {
        // 一時ディレクトリを作成してKDLファイルをコピー
        let config_dir = PathBuf::from(".ichimi_import");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create import directory: {}", e))?;

        let import_dest = config_dir.join("processes.kdl");
        std::fs::copy(file_path, &import_dest)
            .map_err(|e| format!("Failed to copy import file: {}", e))?;

        // KDLファイルを読み込み
        let kdl_persistence = KdlPersistence::new(&config_dir);
        let imported_processes = kdl_persistence
            .get_all_processes()
            .map_err(|e| format!("Failed to read KDL file: {}", e))?;

        // SurrealDBに保存
        for process_config in imported_processes {
            let info = process_config.to_process_info();
            self.save_process(&info).await
                .map_err(|e| format!("Failed to save imported process: {}", e))?;
        }

        // 一時ディレクトリをクリーンアップ
        let _ = std::fs::remove_dir_all(&config_dir);

        tracing::info!("Imported processes from KDL file: {}", file_path);
        Ok(())
    }

    /// Export to JSON file (compatibility)
    pub async fn export_to_file(&self, file_path: &str) -> Result<(), String> {
        let processes = self.load_all_processes().await?;
        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| format!("Failed to serialize processes: {}", e))?;

        std::fs::write(file_path, json)
            .map_err(|e| format!("Failed to write export file: {}", e))?;

        tracing::info!("Exported {} processes to {}", processes.len(), file_path);
        Ok(())
    }

    /// Import from JSON file (compatibility)
    pub async fn import_from_file(&self, file_path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;

        let imported: HashMap<String, ProcessInfo> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse import file: {}", e))?;

        for (_, info) in imported.iter() {
            self.save_process(info).await
                .map_err(|e| format!("Failed to save imported process: {}", e))?;
        }

        tracing::info!("Imported {} processes from {}", imported.len(), file_path);
        Ok(())
    }

    /// Export to default location
    pub async fn export_default(&self) -> Result<String, String> {
        let path = "ichimi_export.json";
        self.export_to_file(path).await?;
        Ok(path.to_string())
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        // For tests only - this will panic if used
        panic!("Default PersistenceManager should not be used in production")
    }
}