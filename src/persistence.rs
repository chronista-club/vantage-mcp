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
        
        // Create table schema - use SCHEMALESS for flexibility
        db.query(r#"
            DEFINE TABLE process SCHEMALESS;
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
        // Use raw SurrealQL with JSON directly embedded
        let args_json = serde_json::to_string(&process_info.args)
            .map_err(|e| format!("Failed to serialize args: {}", e))?;
        let env_json = serde_json::to_string(&process_info.env)
            .map_err(|e| format!("Failed to serialize env: {}", e))?;
        
        let query = format!(
            r#"
            DELETE process:`{}`;
            CREATE process:`{}` SET
                process_id = '{}',
                command = '{}',
                args = {},
                env = {},
                cwd = {},
                state = '{}',
                created_at = time::now(),
                updated_at = time::now()
            RETURN AFTER;
            "#,
            process_info.id,
            process_info.id,
            process_info.id.replace("'", "\\'"),
            process_info.command.replace("'", "\\'"),
            args_json,
            env_json,
            process_info.cwd.as_ref()
                .map(|p| format!("'{}'", p.to_string_lossy().replace("'", "\\'")))
                .unwrap_or("null".to_string()),
            Self::serialize_process_state(&process_info.state).replace("'", "\\'")
        );
        
        self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to save process: {}", e))?;
        
        tracing::debug!("Saved process {} to SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        // Use SurrealQL UPDATE with SET for updates
        let args_json = serde_json::to_string(&process_info.args)
            .map_err(|e| format!("Failed to serialize args: {}", e))?;
        let env_json = serde_json::to_string(&process_info.env)
            .map_err(|e| format!("Failed to serialize env: {}", e))?;
        
        let query = format!(
            r#"
            UPDATE process:`{}` SET
                process_id = '{}',
                command = '{}',
                args = {},
                env = {},
                cwd = {},
                state = '{}',
                updated_at = time::now()
            "#,
            process_info.id,
            process_info.id.replace("'", "\\'"),
            process_info.command.replace("'", "\\'"),
            args_json,
            env_json,
            process_info.cwd.as_ref()
                .map(|p| format!("'{}'", p.to_string_lossy().replace("'", "\\'")))
                .unwrap_or("null".to_string()),
            Self::serialize_process_state(&process_info.state).replace("'", "\\'")
        );
        
        self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to update process: {}", e))?;
        
        tracing::debug!("Updated process {} in SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn delete_process(&self, process_id: &str) -> Result<(), String> {
        let query = format!("DELETE process:`{}`", process_id);
        
        self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to delete process: {}", e))?;
        
        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }
    
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let mut processes = HashMap::new();
        
        // Use SurrealQL to select all processes
        let query = "SELECT * FROM process";
        
        let mut response = self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to load processes: {}", e))?;
        
        let records: Vec<ProcessRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to extract records: {}", e))?;
        
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
    
    /// Query processes with advanced filters using SurrealQL
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        // SurrealQLの強力なクエリ機能を活用
        let query = if filter.is_empty() {
            "SELECT * FROM process ORDER BY created_at DESC".to_string()
        } else {
            format!("SELECT * FROM process WHERE {} ORDER BY created_at DESC", filter)
        };
        
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
    
    /// Get process statistics using SurrealQL aggregation
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let query = r#"
            SELECT 
                count() as total_processes,
                count(state = 'Running') as running_count,
                count(state = 'Stopped') as stopped_count,
                count(state = 'Failed') as failed_count
            FROM process
            GROUP ALL
        "#;
        
        let mut response = self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to get stats: {}", e))?;
        
        let stats: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| format!("Failed to extract stats: {}", e))?;
        
        Ok(stats.into_iter().next().unwrap_or(serde_json::json!({})))
    }
    
    /// Search processes by command or args using SurrealQL full-text search
    pub async fn search_processes(&self, search_term: &str) -> Result<Vec<ProcessInfo>, String> {
        let query = format!(
            r#"
            SELECT * FROM process 
            WHERE command CONTAINS '{}' 
               OR string::join(' ', args) CONTAINS '{}'
            ORDER BY updated_at DESC
            "#,
            search_term, search_term
        );
        
        let mut response = self.db
            .query(&query)
            .await
            .map_err(|e| format!("Failed to search processes: {}", e))?;
        
        let records: Vec<ProcessRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to extract search results: {}", e))?;
        
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
        
        // Use SurrealQL to export all records
        let query = "SELECT * FROM process";
        
        let mut response = self.db
            .query(query)
            .await
            .map_err(|e| format!("Failed to export data: {}", e))?;
        
        let records: Vec<ProcessRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to extract export records: {}", e))?;
        
        // Generate surql statements
        let mut surql_content = String::new();
        
        // Add namespace and database selection
        surql_content.push_str("-- Ichimi Server Export\n");
        surql_content.push_str("-- Generated at: ");
        surql_content.push_str(&chrono::Utc::now().to_rfc3339());
        surql_content.push_str("\n\n");
        surql_content.push_str("USE NS ichimi;\n");
        surql_content.push_str("USE DB processes;\n\n");
        
        // Add schema definition - using SCHEMALESS for flexibility
        surql_content.push_str("-- Define schema\n");
        surql_content.push_str("DEFINE TABLE process SCHEMALESS;\n");
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
        // This is only used in tests or when we can't use async
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Self::new().await.unwrap_or_else(|e| {
                    panic!("Failed to initialize PersistenceManager: {}", e);
                })
            })
        })
    }
}
