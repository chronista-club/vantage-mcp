use crate::db::Database;
use crate::process::types::{ProcessInfo, ProcessState};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessInfoRecord {
    process_id: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<String>,
    #[serde(default)]
    auto_start_on_create: bool,
    #[serde(default)]
    auto_start_on_restore: bool,
}

impl From<&ProcessInfo> for ProcessInfoRecord {
    fn from(info: &ProcessInfo) -> Self {
        Self {
            process_id: info.id.clone(),
            command: info.command.clone(),
            args: info.args.clone(),
            env: info.env.clone(),
            cwd: info.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
            auto_start_on_create: info.auto_start_on_create,
            auto_start_on_restore: info.auto_start_on_restore,
        }
    }
}

impl ProcessInfoRecord {
    fn to_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: self.process_id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.as_ref().map(PathBuf::from),
            state: ProcessState::NotStarted,
            auto_start_on_create: self.auto_start_on_create,
            auto_start_on_restore: self.auto_start_on_restore,
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
        tracing::info!(
            "Attempting to save process {} to SurrealDB",
            process_info.id
        );
        let record = ProcessInfoRecord::from(process_info);
        let client = self.database.client().await;
        
        // 完全なクエリを一度に実行
        let query = r#"
            USE NS ichimi DB main;
            DELETE process WHERE process_id = $process_id;
            CREATE process CONTENT {
                process_id: $process_id,
                command: $command,
                args: $args,
                env: $env,
                cwd: $cwd,
                auto_start_on_create: $auto_start_on_create,
                auto_start_on_restore: $auto_start_on_restore,
                updated_at: time::now()
            };
        "#;

        let mut qr = client
            .query(query)
            .bind(("process_id", record.process_id.clone()))
            .bind(("command", record.command.clone()))
            .bind(("args", record.args.clone()))
            .bind(("env", record.env.clone()))
            .bind(("cwd", record.cwd.clone()))
            .bind(("auto_start_on_create", record.auto_start_on_create))
            .bind(("auto_start_on_restore", record.auto_start_on_restore))
            .await
            .context("Failed to save process to SurrealDB")?;
        
        // 結果を取得してログに出力
        let _: Option<()> = qr.take(0).ok().flatten();  // USE文の結果
        let _: Option<()> = qr.take(1).ok().flatten();  // DELETE文の結果
        let created: Vec<serde_json::Value> = qr.take(2).unwrap_or_default();  // CREATE文の結果
        tracing::debug!("Created {} records", created.len());

        tracing::debug!("Saved process {} to SurrealDB", process_info.id);
        
        // Verify the save by immediately querying it back
        let verify_query = "USE NS ichimi DB main; SELECT * FROM process WHERE process_id = $process_id";
        let mut verify_result = client
            .query(verify_query)
            .bind(("process_id", record.process_id.clone()))
            .await
            .context("Failed to verify save")?;
        
        let _: Option<()> = verify_result.take(0).ok().flatten();  // USE文の結果をスキップ
        let verified: Vec<serde_json::Value> = verify_result.take(1).unwrap_or_default();
        tracing::debug!("Verification query returned {} records", verified.len());
        
        Ok(())
    }

    /// Update a process (alias for save_process)
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<()> {
        self.save_process(process_info).await
    }

    /// Delete a process from SurrealDB
    pub async fn delete_process(&self, process_id: &str) -> Result<()> {
        let client = self.database.client().await;
        
        let query = "USE NS ichimi DB main; DELETE process WHERE process_id = $process_id";
        client
            .query(query)
            .bind(("process_id", process_id.to_string()))
            .await
            .context("Failed to delete process from SurrealDB")?;

        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }

    /// Load all processes from SurrealDB
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let client = self.database.client().await;
        
        let query = "USE NS ichimi DB main; SELECT * FROM process";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query processes: {e}"))?;

        // USE文の結果をスキップして、SELECT文の結果を取得
        let _ = response.take::<Option<()>>(0);  // USE文の結果をスキップ
        
        // ProcessInfoRecord構造体として直接デシリアライズ
        let records: Vec<ProcessInfoRecord> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize process records: {e}"))?;

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
            ("SELECT * FROM process".to_string(), false)
        } else {
            (
                "SELECT * FROM process WHERE command CONTAINS $filter".to_string(),
                true,
            )
        };

        let mut query_builder = client.query(&query);
        if bind_filter {
            query_builder = query_builder.bind(("filter", filter.to_string()));
        }

        let mut response = query_builder
            .await
            .map_err(|e| format!("Failed to query processes: {e}"))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse process records: {e}"))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
    }

    /// Get process statistics from SurrealDB
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let client = self.database.client().await;

        let query = "SELECT count() as total FROM process GROUP ALL";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to get stats: {e}"))?;

        let stats: Option<serde_json::Value> = response
            .take(0)
            .map_err(|e| format!("Failed to parse stats: {e}"))?;

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
            SELECT * FROM process 
            WHERE command CONTAINS $term 
               OR array::any(args, |arg| string::lowercase(arg) CONTAINS $term_lower)
        "#;

        let mut response = client
            .query(query)
            .bind(("term", search_term.to_string()))
            .bind(("term_lower", search_term.to_lowercase()))
            .await
            .map_err(|e| format!("Failed to search processes: {e}"))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse search results: {e}"))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
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
            .map_err(|e| format!("Failed to parse import file: {e}"))?;

        for (_, info) in imported.iter() {
            self.save_process(info)
                .await
                .map_err(|e| format!("Failed to save imported process: {e}"))?;
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
