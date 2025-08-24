use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::process::types::{ProcessInfo, ProcessState};

/// Simple in-memory persistence manager
pub struct PersistenceManager {
    processes: Arc<RwLock<HashMap<String, ProcessInfo>>>,
}

impl PersistenceManager {
    /// Create a new persistence manager with in-memory storage
    pub async fn new() -> Result<Self, String> {
        tracing::info!("Using in-memory persistence");
        Ok(Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        let mut processes = self.processes.write().await;
        processes.insert(process_info.id.clone(), process_info.clone());
        tracing::debug!("Saved process {} to memory", process_info.id);
        Ok(())
    }
    
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<(), String> {
        let mut processes = self.processes.write().await;
        processes.insert(process_info.id.clone(), process_info.clone());
        tracing::debug!("Updated process {} in memory", process_info.id);
        Ok(())
    }
    
    pub async fn delete_process(&self, process_id: &str) -> Result<(), String> {
        let mut processes = self.processes.write().await;
        processes.remove(process_id);
        tracing::debug!("Deleted process {} from memory", process_id);
        Ok(())
    }
    
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let processes = self.processes.read().await;
        let mut result = HashMap::new();
        
        for (id, info) in processes.iter() {
            let mut cloned = info.clone();
            cloned.state = ProcessState::NotStarted; // Reset state on startup
            result.insert(id.clone(), cloned);
        }
        
        tracing::info!("Loaded {} processes from memory", result.len());
        Ok(result)
    }
    
    /// Query processes with simple filters
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let processes = self.processes.read().await;
        
        if filter.is_empty() {
            Ok(processes.values().cloned().collect())
        } else {
            // Simple filter by command name
            Ok(processes
                .values()
                .filter(|p| p.command.contains(filter))
                .cloned()
                .collect())
        }
    }
    
    /// Get process statistics
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let processes = self.processes.read().await;
        
        let mut running_count = 0;
        let mut stopped_count = 0;
        let mut failed_count = 0;
        
        for process in processes.values() {
            match &process.state {
                ProcessState::Running { .. } => running_count += 1,
                ProcessState::Stopped { .. } => stopped_count += 1,
                ProcessState::Failed { .. } => failed_count += 1,
                ProcessState::NotStarted => {}
            }
        }
        
        Ok(serde_json::json!({
            "total_processes": processes.len(),
            "running_count": running_count,
            "stopped_count": stopped_count,
            "failed_count": failed_count
        }))
    }
    
    /// Search processes by command or args
    pub async fn search_processes(&self, search_term: &str) -> Result<Vec<ProcessInfo>, String> {
        let processes = self.processes.read().await;
        let search_lower = search_term.to_lowercase();
        
        Ok(processes
            .values()
            .filter(|p| {
                p.command.to_lowercase().contains(&search_lower) ||
                p.args.iter().any(|arg| arg.to_lowercase().contains(&search_lower))
            })
            .cloned()
            .collect())
    }
    
    /// Export to JSON file
    pub async fn export_to_file(&self, file_path: &str) -> Result<(), String> {
        let processes = self.processes.read().await;
        let json = serde_json::to_string_pretty(&*processes)
            .map_err(|e| format!("Failed to serialize processes: {}", e))?;
        
        std::fs::write(file_path, json)
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        tracing::info!("Exported {} processes to {}", processes.len(), file_path);
        Ok(())
    }
    
    /// Import from JSON file
    pub async fn import_from_file(&self, file_path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;
        
        let imported: HashMap<String, ProcessInfo> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse import file: {}", e))?;
        
        let mut processes = self.processes.write().await;
        *processes = imported;
        
        tracing::info!("Imported {} processes from {}", processes.len(), file_path);
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
        // Simple default implementation for tests
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}