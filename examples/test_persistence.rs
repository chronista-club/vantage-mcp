use ichimi_server::persistence::PersistenceManager;
use ichimi_server::process::types::{ProcessInfo, ProcessState};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create persistence manager
    let persistence = PersistenceManager::new().await?;

    // Create test process
    let process_info = ProcessInfo {
        id: "test-example".to_string(),
        command: "echo".to_string(),
        args: vec!["hello".to_string(), "world".to_string()],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
    };

    println!("Saving process: {:?}", process_info);

    // Save process
    persistence.save_process(&process_info).await?;
    println!("Process saved successfully");

    // Debug: Query directly
    println!("\nDebug: Querying database directly...");

    // Load all processes
    let loaded = persistence.load_all_processes().await?;
    println!("Loaded {} processes", loaded.len());

    if let Some(proc) = loaded.get("test-example") {
        println!("Loaded process: {:?}", proc);
        println!("Args: {:?}", proc.args);
    }

    // Export to file
    let export_path = "/tmp/test_export.surql";
    persistence.export_to_file(export_path).await?;
    println!("Exported to {}", export_path);

    // Read and print the export file
    let content = std::fs::read_to_string(export_path)?;
    println!("\nExport file content:\n{}", content);

    Ok(())
}
