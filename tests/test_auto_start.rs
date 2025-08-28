use ichimi_server::process::ProcessManager;
use std::collections::HashMap;

#[tokio::test]
async fn test_update_process_config_auto_start() {
    let manager = ProcessManager::new().await;
    
    // Create a process with auto_start = false
    manager
        .create_process(
            "auto-start-test".to_string(),
            "echo".to_string(),
            vec!["test".to_string()],
            HashMap::new(),
            None,
        )
        .await
        .expect("Failed to create process");
    
    // Get initial status
    let status_before = manager
        .get_process_status("auto-start-test".to_string())
        .await
        .expect("Failed to get process status");
    assert_eq!(status_before.info.auto_start, false);
    
    // Update auto_start to true
    manager
        .update_process_config("auto-start-test".to_string(), Some(true))
        .await
        .expect("Failed to update process config");
    
    // Verify auto_start was updated
    let status_after = manager
        .get_process_status("auto-start-test".to_string())
        .await
        .expect("Failed to get process status");
    assert_eq!(status_after.info.auto_start, true);
    
    // Clean up
    manager
        .remove_process("auto-start-test".to_string())
        .await
        .expect("Failed to remove process");
}

#[tokio::test]
async fn test_update_config_on_running_process() {
    let manager = ProcessManager::new().await;
    
    // Create and start a process
    manager
        .create_process(
            "running-auto-test".to_string(),
            "sleep".to_string(),
            vec!["10".to_string()],
            HashMap::new(),
            None,
        )
        .await
        .expect("Failed to create process");
    
    let pid = manager
        .start_process("running-auto-test".to_string())
        .await
        .expect("Failed to start process");
    assert!(pid > 0);
    
    // Update auto_start while running
    manager
        .update_process_config("running-auto-test".to_string(), Some(true))
        .await
        .expect("Failed to update config while running");
    
    // Verify update was successful
    let status = manager
        .get_process_status("running-auto-test".to_string())
        .await
        .expect("Failed to get process status");
    assert_eq!(status.info.auto_start, true);
    assert!(matches!(
        status.info.state,
        ichimi_server::process::types::ProcessState::Running { .. }
    ));
    
    // Stop and clean up
    manager
        .stop_process("running-auto-test".to_string(), Some(1000))
        .await
        .expect("Failed to stop process");
    
    manager
        .remove_process("running-auto-test".to_string())
        .await
        .expect("Failed to remove process");
}

#[tokio::test]
async fn test_auto_start_persistence() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("auto_start_test.json");
    let export_path_str = export_path.to_str().unwrap().to_string();
    
    // Create manager and add process with auto_start
    {
        let manager = ProcessManager::new().await;
        
        manager
            .create_process(
                "persist-auto-test".to_string(),
                "echo".to_string(),
                vec!["hello".to_string()],
                HashMap::new(),
                None,
            )
            .await
            .expect("Failed to create process");
        
        // Update auto_start to true
        manager
            .update_process_config("persist-auto-test".to_string(), Some(true))
            .await
            .expect("Failed to update config");
        
        // Export to file
        manager
            .export_processes(Some(export_path_str.clone()))
            .await
            .expect("Failed to export processes");
    }
    
    // Import in new manager and verify auto_start is preserved
    {
        let manager = ProcessManager::new().await;
        
        manager
            .import_processes(&export_path_str)
            .await
            .expect("Failed to import processes");
        
        let processes = manager.list_processes(None).await;
        let process = processes
            .iter()
            .find(|p| p.id == "persist-auto-test")
            .expect("Process not found after import");
        
        assert_eq!(process.auto_start, true, "auto_start not preserved after import");
    }
}

#[tokio::test]
async fn test_update_nonexistent_process() {
    let manager = ProcessManager::new().await;
    
    // Try to update non-existent process
    let result = manager
        .update_process_config("nonexistent".to_string(), Some(true))
        .await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_toggle_auto_start_multiple_times() {
    let manager = ProcessManager::new().await;
    
    // Create process
    manager
        .create_process(
            "toggle-test".to_string(),
            "echo".to_string(),
            vec!["test".to_string()],
            HashMap::new(),
            None,
        )
        .await
        .expect("Failed to create process");
    
    // Toggle auto_start multiple times
    for expected in [true, false, true, false] {
        manager
            .update_process_config("toggle-test".to_string(), Some(expected))
            .await
            .expect("Failed to update config");
        
        let status = manager
            .get_process_status("toggle-test".to_string())
            .await
            .expect("Failed to get status");
        
        assert_eq!(
            status.info.auto_start, expected,
            "auto_start should be {} after toggle",
            expected
        );
    }
    
    // Clean up
    manager
        .remove_process("toggle-test".to_string())
        .await
        .expect("Failed to remove process");
}

#[tokio::test]
async fn test_auto_start_with_kdl_persistence() {
    // This test verifies that auto_start is correctly saved in KDL format
    let manager = ProcessManager::new().await;
    
    // Create a process with auto_start enabled
    manager
        .create_process(
            "kdl-auto-test".to_string(),
            "python".to_string(),
            vec!["-m".to_string(), "http.server".to_string()],
            HashMap::new(),
            Some(std::path::PathBuf::from("/tmp")),
        )
        .await
        .expect("Failed to create process");
    
    // Enable auto_start
    manager
        .update_process_config("kdl-auto-test".to_string(), Some(true))
        .await
        .expect("Failed to enable auto_start");
    
    // Wait a bit for persistence
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Check if the KDL file contains the auto_start setting
    let kdl_path = std::path::PathBuf::from(".ichimi/processes.kdl");
    if kdl_path.exists() {
        let content = std::fs::read_to_string(&kdl_path)
            .expect("Failed to read KDL file");
        
        assert!(
            content.contains("kdl-auto-test"),
            "KDL file should contain the process ID"
        );
        assert!(
            content.contains("auto_start") && content.contains("true"),
            "KDL file should contain auto_start true"
        );
    }
    
    // Clean up
    manager
        .remove_process("kdl-auto-test".to_string())
        .await
        .expect("Failed to remove process");
}