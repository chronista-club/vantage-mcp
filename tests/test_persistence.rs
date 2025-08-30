use ichimi_server::persistence::PersistenceManager;
use ichimi_server::process::types::{ProcessInfo, ProcessState};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_save_and_load_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Create test process
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let process_info = ProcessInfo {
        id: "test-process".to_string(),
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        env,
        cwd: Some(PathBuf::from("/tmp")),
        state: ProcessState::NotStarted,
        auto_start: false,
    };

    // Save process
    persistence.save_process(&process_info).await.unwrap();

    // Load all processes
    let loaded = persistence.load_all_processes().await.unwrap();

    assert_eq!(loaded.len(), 1);
    assert!(loaded.contains_key("test-process"));

    let loaded_process = &loaded["test-process"];
    assert_eq!(loaded_process.id, "test-process");
    assert_eq!(loaded_process.command, "echo");
    assert_eq!(loaded_process.args, vec!["hello"]);
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_update_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    let mut process_info = ProcessInfo {
        id: "update-test".to_string(),
        command: "python".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start: false,
    };

    // Save initial process
    persistence.save_process(&process_info).await.unwrap();

    // Update process
    process_info.command = "python3".to_string();
    process_info.args = vec!["-m".to_string(), "http.server".to_string()];
    persistence.update_process(&process_info).await.unwrap();

    // Load and verify
    let loaded = persistence.load_all_processes().await.unwrap();
    let loaded_process = &loaded["update-test"];

    assert_eq!(loaded_process.command, "python3");
    assert_eq!(loaded_process.args.len(), 2);
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_delete_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    let process_info = ProcessInfo {
        id: "delete-test".to_string(),
        command: "ls".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start: false,
    };

    // Save and delete
    persistence.save_process(&process_info).await.unwrap();
    persistence.delete_process("delete-test").await.unwrap();

    // Verify deleted
    let loaded = persistence.load_all_processes().await.unwrap();
    assert!(!loaded.contains_key("delete-test"));
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_export_import() {
    // Create temp directory for test files
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("test_export.surql");
    let export_path_str = export_path.to_str().unwrap();

    // Create first persistence instance and add data
    {
        let persistence = PersistenceManager::new().await.unwrap();

        // Add multiple processes
        for i in 1..=3 {
            let process_info = ProcessInfo {
                id: format!("process-{i}"),
                command: format!("cmd-{i}"),
                args: vec![format!("arg-{}", i)],
                env: HashMap::new(),
                cwd: Some(PathBuf::from(format!("/path/{i}"))),
                state: ProcessState::NotStarted,
                auto_start: false,
            };
            persistence.save_process(&process_info).await.unwrap();
        }

        // Export to file
        persistence.export_to_file(export_path_str).await.unwrap();
    }

    // Verify export file exists
    assert!(export_path.exists());

    // Create new persistence instance and import
    {
        let persistence = PersistenceManager::new().await.unwrap();

        // Import from file
        persistence.import_from_file(export_path_str).await.unwrap();

        // Load and verify all processes were imported
        let loaded = persistence.load_all_processes().await.unwrap();

        assert_eq!(loaded.len(), 3);
        assert!(loaded.contains_key("process-1"));
        assert!(loaded.contains_key("process-2"));
        assert!(loaded.contains_key("process-3"));

        // Verify process details
        let process_1 = &loaded["process-1"];
        assert_eq!(process_1.command, "cmd-1");
        assert_eq!(process_1.args, vec!["arg-1"]);
        assert_eq!(process_1.cwd, Some(PathBuf::from("/path/1")));
    }
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_export_default_path() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Add a process
    let process_info = ProcessInfo {
        id: "default-export-test".to_string(),
        command: "test".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start: false,
    };
    persistence.save_process(&process_info).await.unwrap();

    // Export to default location
    let export_path = persistence.export_default().await.unwrap();

    // Verify file was created
    assert!(std::path::Path::new(&export_path).exists());

    // Clean up
    let _ = std::fs::remove_file(&export_path);
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_import_nonexistent_file() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Try to import non-existent file
    let result = persistence
        .import_from_file("/nonexistent/file.surql")
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_process_state_serialization() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Test different process states
    let states = vec![
        ProcessState::NotStarted,
        ProcessState::Running {
            pid: 12345,
            started_at: chrono::Utc::now(),
        },
        ProcessState::Stopped {
            exit_code: Some(0),
            stopped_at: chrono::Utc::now(),
        },
        ProcessState::Failed {
            error: "Test error".to_string(),
            failed_at: chrono::Utc::now(),
        },
    ];

    for (i, state) in states.into_iter().enumerate() {
        let process_info = ProcessInfo {
            id: format!("state-test-{i}"),
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            state: state.clone(),
            auto_start: false,
        };

        persistence.save_process(&process_info).await.unwrap();

        // Note: When loading, state is reset to NotStarted
        // This is by design - processes don't persist their runtime state
        let loaded = persistence.load_all_processes().await.unwrap();
        assert!(loaded.contains_key(&format!("state-test-{i}")));
        assert_eq!(
            loaded[&format!("state-test-{i}")].state,
            ProcessState::NotStarted
        );
    }
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_export_empty_database() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("empty_export.surql");
    let export_path_str = export_path.to_str().unwrap();

    let persistence = PersistenceManager::new().await.unwrap();

    // Export empty database
    persistence.export_to_file(export_path_str).await.unwrap();

    // Verify file exists and contains schema
    assert!(export_path.exists());

    let content = std::fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("DEFINE TABLE process"));
    assert!(content.contains("USE NS ichimi"));
}

#[tokio::test]
#[ignore] // TODO: Fix after implementing proper persistence
async fn test_multiple_import_export_cycles() {
    let temp_dir = TempDir::new().unwrap();

    // Initial data
    let process_info = ProcessInfo {
        id: "cycle-test".to_string(),
        command: "cycle".to_string(),
        args: vec!["test".to_string()],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start: false,
    };

    // First cycle
    {
        let persistence = PersistenceManager::new().await.unwrap();
        persistence.save_process(&process_info).await.unwrap();

        let export_path = temp_dir.path().join("cycle1.surql");
        persistence
            .export_to_file(export_path.to_str().unwrap())
            .await
            .unwrap();
    }

    // Second cycle - import and re-export
    {
        let persistence = PersistenceManager::new().await.unwrap();

        let import_path = temp_dir.path().join("cycle1.surql");
        persistence
            .import_from_file(import_path.to_str().unwrap())
            .await
            .unwrap();

        let export_path = temp_dir.path().join("cycle2.surql");
        persistence
            .export_to_file(export_path.to_str().unwrap())
            .await
            .unwrap();
    }

    // Third cycle - verify data integrity
    {
        let persistence = PersistenceManager::new().await.unwrap();

        let import_path = temp_dir.path().join("cycle2.surql");
        persistence
            .import_from_file(import_path.to_str().unwrap())
            .await
            .unwrap();

        let loaded = persistence.load_all_processes().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded["cycle-test"].command, "cycle");
        assert_eq!(loaded["cycle-test"].args, vec!["test"]);
    }
}
