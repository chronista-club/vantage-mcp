use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem};
use crate::process::types::{ProcessInfo, ProcessState};
use crate::model::{ModelDb, Process, Schema};

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
        
        // Create table schema using Schema helper
        Schema::define_process_table(&db).await?;
        
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
    
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        let process = Process::from_process_info(process_info);
        let model_db = ModelDb::new(&self.db);
        model_db.save(&process).await?;
        tracing::debug!("Saved process {} to SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        let process = Process::from_process_info(process_info);
        let model_db = ModelDb::new(&self.db);
        model_db.update(&process).await?;
        tracing::debug!("Updated process {} in SurrealDB", process_info.id);
        Ok(())
    }
    
    pub async fn delete_process(&self, process_id: &str) -> Result<(), String> {
        let model_db = ModelDb::new(&self.db);
        model_db.delete::<Process>(process_id).await?;
        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }
    
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let mut processes = HashMap::new();
        
        let model_db = ModelDb::new(&self.db);
        let records = model_db.find_all::<Process>().await?;
        
        for record in records {
            let mut info = record.to_process_info();
            info.state = ProcessState::NotStarted; // Reset state on startup
            processes.insert(info.id.clone(), info);
        }
        
        tracing::info!("Loaded {} processes from SurrealDB", processes.len());
        Ok(processes)
    }
    
    /// Query processes with advanced filters using SurrealQL
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let model_db = ModelDb::new(&self.db);
        let records = if filter.is_empty() {
            model_db.find_all::<Process>().await?
        } else {
            model_db.find_by::<Process>(filter).await?
        };
        
        let processes = records.into_iter()
            .map(|record| record.to_process_info())
            .collect();
        
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
        let filter = format!(
            "command CONTAINS '{}' OR string::join(' ', args) CONTAINS '{}'",
            search_term, search_term
        );
        
        let model_db = ModelDb::new(&self.db);
        let records = model_db.find_by::<Process>(&filter).await?;
        
        let processes = records.into_iter()
            .map(|record| record.to_process_info())
            .collect();
        
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
        
        // Use ModelDb to get all records
        let model_db = ModelDb::new(&self.db);
        let records = model_db.find_all::<Process>().await?;
        
        // Generate surql statements
        let mut surql_content = String::new();
        
        // Add namespace and database selection
        surql_content.push_str("-- Ichimi Server Export\n");
        surql_content.push_str("-- Generated at: ");
        surql_content.push_str(&chrono::Utc::now().to_rfc3339());
        surql_content.push_str("\n\n");
        surql_content.push_str("USE NS ichimi;\n");
        surql_content.push_str("USE DB processes;\n\n");
        
        // Add schema definition with SCHEMAFULL
        surql_content.push_str("-- Define schema\n");
        surql_content.push_str("DEFINE TABLE process SCHEMALESS;\n");  // Use SCHEMALESS for import compatibility
        surql_content.push_str("DEFINE INDEX idx_process_id ON TABLE process COLUMNS process_id UNIQUE;\n\n");
        
        // Add data
        surql_content.push_str("-- Process data\n");
        for record in &records {
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
