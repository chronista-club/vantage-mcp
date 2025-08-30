use ichimi_server::process::ProcessManager;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_process_manager_export_import() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("manager_export.surql");
    let export_path_str = export_path.to_str().unwrap().to_string();

    // Create first manager and add processes
    let manager1 = ProcessManager::new().await;

    // Create multiple processes
    for i in 1..=3 {
        manager1
            .create_process(
                format!("manager-test-{i}"),
                "echo".to_string(),
                vec![format!("Process {}", i)],
                HashMap::new(),
                None,
            )
            .await
            .unwrap();
    }

    // Export processes
    let exported_path = manager1
        .export_processes(Some(export_path_str.clone()))
        .await
        .unwrap();
    assert_eq!(exported_path, export_path_str);
    assert!(export_path.exists());

    // Create new manager and import
    let manager2 = ProcessManager::new().await;

    // Initially should be empty (or with auto-imported processes)
    // Import our test processes
    manager2.import_processes(&export_path_str).await.unwrap();

    // Verify all processes were imported
    let processes = manager2.list_processes(None).await;

    // Should have at least our 3 test processes
    let test_processes: Vec<_> = processes
        .iter()
        .filter(|p| p.id.starts_with("manager-test-"))
        .collect();

    assert_eq!(test_processes.len(), 3);

    // Verify process details
    for i in 1..=3 {
        let process = test_processes
            .iter()
            .find(|p| p.id == format!("manager-test-{i}"))
            .unwrap_or_else(|| panic!("Process manager-test-{i} not found"));

        assert_eq!(process.command, "echo");
        assert_eq!(process.args, vec![format!("Process {i}")]);
    }
}

#[tokio::test]
async fn test_process_lifecycle_with_persistence() {
    let manager = ProcessManager::new().await;

    // Create a process
    manager
        .create_process(
            "lifecycle-test".to_string(),
            "sleep".to_string(),
            vec!["1".to_string()],
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    // Export to temp file
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("lifecycle.surql");
    manager
        .export_processes(Some(export_path.to_str().unwrap().to_string()))
        .await
        .unwrap();

    // Start the process
    let pid = manager
        .start_process("lifecycle-test".to_string())
        .await
        .unwrap();
    assert!(pid > 0);

    // Get status while running
    let status = manager
        .get_process_status("lifecycle-test".to_string())
        .await
        .unwrap();
    assert!(matches!(
        status.info.state,
        ichimi_server::process::types::ProcessState::Running { .. }
    ));

    // Wait for process to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Stop the process (it should already be done)
    let _ = manager
        .stop_process("lifecycle-test".to_string(), Some(1000))
        .await;

    // Remove the process
    manager
        .remove_process("lifecycle-test".to_string())
        .await
        .unwrap();

    // Verify it's removed
    let processes = manager.list_processes(None).await;
    assert!(!processes.iter().any(|p| p.id == "lifecycle-test"));
}

#[tokio::test]
async fn test_export_with_environment_variables() {
    let manager = ProcessManager::new().await;

    // Create process with environment variables
    let mut env = HashMap::new();
    env.insert("TEST_VAR_1".to_string(), "value1".to_string());
    env.insert("TEST_VAR_2".to_string(), "value2".to_string());
    env.insert("PATH_OVERRIDE".to_string(), "/custom/path".to_string());

    manager
        .create_process(
            "env-test".to_string(),
            "printenv".to_string(),
            vec![],
            env.clone(),
            Some(PathBuf::from("/tmp")),
        )
        .await
        .unwrap();

    // Export and import
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("env_test.surql");
    let export_path_str = export_path.to_str().unwrap().to_string();

    manager
        .export_processes(Some(export_path_str.clone()))
        .await
        .unwrap();

    // Create new manager and import
    let manager2 = ProcessManager::new().await;
    manager2.import_processes(&export_path_str).await.unwrap();

    // Verify environment variables were preserved
    let processes = manager2.list_processes(None).await;
    let env_process = processes
        .iter()
        .find(|p| p.id == "env-test")
        .expect("env-test process not found");

    assert_eq!(env_process.env.len(), 3);
    assert_eq!(
        env_process.env.get("TEST_VAR_1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        env_process.env.get("TEST_VAR_2"),
        Some(&"value2".to_string())
    );
    assert_eq!(
        env_process.env.get("PATH_OVERRIDE"),
        Some(&"/custom/path".to_string())
    );
    assert_eq!(env_process.cwd, Some(PathBuf::from("/tmp")));
}

#[tokio::test]
async fn test_default_export_path() {
    let manager = ProcessManager::new().await;

    // Create a test process
    manager
        .create_process(
            "default-path-test".to_string(),
            "ls".to_string(),
            vec!["-la".to_string()],
            HashMap::new(),
            None,
        )
        .await
        .unwrap();

    // Export to default path
    let export_path = manager.export_processes(None).await.unwrap();

    // Verify file exists at default location
    assert!(std::path::Path::new(&export_path).exists());
    assert!(export_path.contains("ichimi"));

    // Clean up
    let _ = std::fs::remove_file(&export_path);
}

#[tokio::test]
async fn test_import_error_handling() {
    let manager = ProcessManager::new().await;

    // Try to import non-existent file
    let result = manager
        .import_processes("/this/file/does/not/exist.surql")
        .await;
    assert!(result.is_err());

    // Try to import invalid file
    let temp_dir = TempDir::new().unwrap();
    let invalid_file = temp_dir.path().join("invalid.surql");
    std::fs::write(&invalid_file, "INVALID SURQL CONTENT").unwrap();

    let result = manager
        .import_processes(invalid_file.to_str().unwrap())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_export() {
    let manager = ProcessManager::new().await;

    // Create multiple processes
    for i in 1..=5 {
        manager
            .create_process(
                format!("concurrent-{i}"),
                "echo".to_string(),
                vec![i.to_string()],
                HashMap::new(),
                None,
            )
            .await
            .unwrap();
    }

    let temp_dir = TempDir::new().unwrap();

    // Perform multiple concurrent exports
    let mut handles = vec![];
    for i in 1..=3 {
        let manager_clone = manager.clone();
        let export_path = temp_dir.path().join(format!("concurrent_{i}.surql"));
        let export_path_str = export_path.to_str().unwrap().to_string();

        let handle =
            tokio::spawn(
                async move { manager_clone.export_processes(Some(export_path_str)).await },
            );
        handles.push(handle);
    }

    // Wait for all exports to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all export files exist and contain the same data
    for i in 1..=3 {
        let export_path = temp_dir.path().join(format!("concurrent_{i}.surql"));
        assert!(export_path.exists());

        let content = std::fs::read_to_string(&export_path).unwrap();
        assert!(content.contains("concurrent-1"));
        assert!(content.contains("concurrent-5"));
    }
}
