use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::sql::{Thing, Datetime};
use serde::{Serialize, Deserialize};
use crate::process::types::{ProcessInfo, ProcessState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessRecord {
    pub id: Thing,
    pub process_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub state: String,  // Serialized ProcessState
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

pub struct PersistenceManager {
    db: Surreal<Db>,
}

impl PersistenceManager {
    /// Create a new persistence manager with in-memory database
    pub async fn new() -> Result<Self, String> {
        tracing::info!("Using SurrealDB with in-memory persistence");
        let db = Surreal::new::<Mem>(())
            .await
            .map_err(|e| format!("Failed to create in-memory database: {}", e))?;
        
        // Initialize database
        db.use_ns("ichimi").use_db("processes").await
            .map_err(|e| format!("Failed to select namespace/database: {}", e))?;
        
        // Create table schema
        db.query(r#"
            DEFINE TABLE process SCHEMAFULL;
            DEFINE FIELD process_id ON TABLE process TYPE string;
            DEFINE FIELD command ON TABLE process TYPE string;
            DEFINE FIELD args ON TABLE process TYPE array;
            DEFINE FIELD env ON TABLE process TYPE object;
            DEFINE FIELD cwd ON TABLE process TYPE option<string>;
            DEFINE FIELD state ON TABLE process TYPE string;
            DEFINE FIELD created_at ON TABLE process TYPE datetime;
            DEFINE FIELD updated_at ON TABLE process TYPE datetime;
            DEFINE INDEX idx_process_id ON TABLE process COLUMNS process_id UNIQUE;
        "#)
        .await
        .map_err(|e| format!("Failed to create schema: {}", e))?;
        
        let manager = Self { db };
        
        // Auto-import if file exists
        if let Ok(import_file) = std::env::var("ICHIMI_IMPORT_FILE") {
            if let Err(e) = manager.import_from_file(&import_file).await {
                tracing::warn!("Failed to auto-import from {}: {}", import_file, e);
            } else {
                tracing::info!("Auto-imported data from {}", import_file);
            }
        } else {
            // Try default import file
            let default_file = Self::get_default_export_path();
            if default_file.exists() {
                if let Err(e) = manager.import_from_file(default_file.to_str().unwrap()).await {
                    tracing::debug!("No default import file or failed to import: {}", e);
                } else {
                    tracing::info!("Auto-imported data from default file");
                }
            }
        }
        
        Ok(manager)
    }
    
    fn get_data_directory() -> PathBuf {
        if let Ok(custom_dir) = std::env::var("ICHIMI_DATA_DIR") {
            return PathBuf::from(custom_dir);
        }
        
        if let Some(home_dir) = dirs::home_dir() {
            return home_dir.join(".ichimi").join("data");
        }
        
        PathBuf::from("data")
    }
    
    fn get_default_export_path() -> PathBuf {
        Self::get_data_directory().join("ichimi_export.surql")
    }
    
    fn serialize_process_state(state: &ProcessState) -> String {
        serde_json::to_string(state).unwrap_or_else(|_| "NotStarted".to_string())
    }
    
    fn deserialize_process_state(state_str: &str) -> ProcessState {
        serde_json::from_str(state_str).unwrap_or(ProcessState::NotStarted)
    }
    
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        // Use UPDATE instead of CREATE to ensure all fields are properly set
        let query = format!(
            r#"
            UPDATE process:`{}` SET
                process_id = $process_id,
                command = $command,
                args = $args,
                env = $env,
                cwd = $cwd,
                state = $state,
                created_at = $created_at,
                updated_at = $updated_at
            "#,
            process_info.id
        );
        
        let mut response = self.db
            .query(query)
            .bind(("process_id", process_info.id.clone()))
            .bind(("command", process_info.command.clone()))
            .bind(("args", process_info.args.clone()))
            .bind(("env", process_info.env.clone()))
            .bind(("cwd", process_info.cwd.as_ref().map(|p| p.to_string_lossy().to_string())))
            .bind(("state", Self::serialize_process_state(&process_info.state)))
            .bind(("created_at", Datetime::from(chrono::Utc::now())))
            .bind(("updated_at", Datetime::from(chrono::Utc::now())))
            .await
            .map_err(|e| format!("Failed to save process: {}", e))?;
        
        tracing::debug!("Saved process {} to SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        // Use the same UPDATE query as save_process
        let query = format!(
            r#"
            UPDATE process:`{}` SET
                process_id = $process_id,
                command = $command,
                args = $args,
                env = $env,
                cwd = $cwd,
                state = $state,
                updated_at = $updated_at
            "#,
            process_info.id
        );
        
        let mut response = self.db
            .query(query)
            .bind(("process_id", process_info.id.clone()))
            .bind(("command", process_info.command.clone()))
            .bind(("args", process_info.args.clone()))
            .bind(("env", process_info.env.clone()))
            .bind(("cwd", process_info.cwd.as_ref().map(|p| p.to_string_lossy().to_string())))
            .bind(("state", Self::serialize_process_state(&process_info.state)))
            .bind(("updated_at", Datetime::from(chrono::Utc::now())))
            .await
            .map_err(|e| format!("Failed to update process: {}", e))?;
        
        tracing::debug!("Updated process {} in SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn delete_process(&self, process_id: &str) -> Result<(), String> {
        let _: Option<ProcessRecord> = self.db
            .delete(("process", process_id))
            .await
            .map_err(|e| format!("Failed to delete process: {}", e))?;
        
        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }
    
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let mut processes = HashMap::new();
        
        let records: Vec<ProcessRecord> = self.db
            .select("process")
            .await
            .map_err(|e| format!("Failed to load processes: {}", e))?;
        
        for record in records {
            let info = ProcessInfo {
                id: record.process_id.clone(),
                command: record.command,
                args: record.args,
                env: record.env,
                cwd: record.cwd.map(PathBuf::from),
                state: ProcessState::NotStarted, // Reset state on startup
            };
            processes.insert(record.process_id, info);
        }
        
        tracing::info!("Loaded {} processes from SurrealDB", processes.len());
        Ok(processes)
    }
    
    /// Query processes with advanced filters
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let query = format!(
            "SELECT * FROM process WHERE {}",
            if filter.is_empty() { "true" } else { filter }
        );
        
        let mut response = self.db
            .query(&query)
            .await
            .map_err(|e| format!("Failed to query processes: {}", e))?;
        
        let records: Vec<ProcessRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to extract query results: {}", e))?;
        
        let processes = records.into_iter().map(|record| {
            ProcessInfo {
                id: record.process_id,
                command: record.command,
                args: record.args,
                env: record.env,
                cwd: record.cwd.map(PathBuf::from),
                state: Self::deserialize_process_state(&record.state),
            }
        }).collect();
        
        Ok(processes)
    }
    
    /// Export database to surql file
    pub async fn export_to_file(&self, file_path: &str) -> Result<(), String> {
        // Ensure directory exists
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create export directory: {}", e))?;
            }
        }
        
        // Export all records from process table
        let records: Vec<ProcessRecord> = self.db
            .select("process")
            .await
            .map_err(|e| format!("Failed to export data: {}", e))?;
        
        // Generate surql statements
        let mut surql_content = String::new();
        
        // Add namespace and database selection
        surql_content.push_str("-- Ichimi Server Export\n");
        surql_content.push_str("-- Generated at: ");
        surql_content.push_str(&chrono::Utc::now().to_rfc3339());
        surql_content.push_str("\n\n");
        surql_content.push_str("USE NS ichimi;\n");
        surql_content.push_str("USE DB processes;\n\n");
        
        // Add schema definition
        surql_content.push_str("-- Define schema\n");
        surql_content.push_str("DEFINE TABLE process SCHEMAFULL;\n");
        surql_content.push_str("DEFINE FIELD process_id ON TABLE process TYPE string;\n");
        surql_content.push_str("DEFINE FIELD command ON TABLE process TYPE string;\n");
        surql_content.push_str("DEFINE FIELD args ON TABLE process TYPE array;\n");
        surql_content.push_str("DEFINE FIELD env ON TABLE process TYPE object;\n");
        surql_content.push_str("DEFINE FIELD cwd ON TABLE process TYPE option<string>;\n");
        surql_content.push_str("DEFINE FIELD state ON TABLE process TYPE string;\n");
        surql_content.push_str("DEFINE FIELD created_at ON TABLE process TYPE datetime;\n");
        surql_content.push_str("DEFINE FIELD updated_at ON TABLE process TYPE datetime;\n");
        surql_content.push_str("DEFINE INDEX idx_process_id ON TABLE process COLUMNS process_id UNIQUE;\n\n");
        
        // Add data
        surql_content.push_str("-- Process data\n");
        for record in records {
            let json = serde_json::to_string(&record)
                .map_err(|e| format!("Failed to serialize record: {}", e))?;
            surql_content.push_str(&format!(
                "CREATE process:`{}` CONTENT {};\n",
                record.process_id, json
            ));
        }
        
        // Write to file
        fs::write(file_path, surql_content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        tracing::info!("Exported database to {}", file_path);
        Ok(())
    }
    
    /// Import database from surql file
    pub async fn import_from_file(&self, file_path: &str) -> Result<(), String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("Import file does not exist: {}", file_path));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;
        
        // Execute the surql content
        self.db
            .query(content)
            .await
            .map_err(|e| format!("Failed to import data: {}", e))?;
        
        tracing::info!("Imported database from {}", file_path);
        Ok(())
    }
    
    /// Export to default location
    pub async fn export_default(&self) -> Result<String, String> {
        let path = Self::get_default_export_path();
        let path_str = path.to_string_lossy().to_string();
        self.export_to_file(&path_str).await?;
        Ok(path_str)
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        // Use blocking version for default implementation
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                Self::new().await.unwrap_or_else(|e| {
                    panic!("Failed to initialize PersistenceManager: {}", e);
                })
            })
        })
        .join()
        .unwrap()
    }
}
