/// Unit tests for persistence layer (PersistenceManager)
///
/// Tests basic CRUD operations for process persistence
use ichimi_persistence::{PersistenceManager, ProcessInfo, ProcessState, ProcessStatus};
use std::collections::HashMap;

#[tokio::test]
async fn test_save_and_load_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Create test process
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let process_info = ProcessInfo {
        id: None,
        process_id: "test-process".to_string(),
        name: "test-process".to_string(),
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        env,
        cwd: Some("/tmp".to_string()),
        status: ProcessStatus {
            state: ProcessState::NotStarted,
            pid: None,
            exit_code: None,
            started_at: None,
            stopped_at: None,
            error: None,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
        auto_start: false,
    };

    // Save process
    persistence.save_process(&process_info).await.unwrap();

    // Load all processes
    let loaded = persistence.load_all_processes().await.unwrap();

    assert_eq!(loaded.len(), 1);
    assert!(loaded.contains_key("test-process"));

    let loaded_process = &loaded["test-process"];
    assert_eq!(loaded_process.process_id, "test-process");
    assert_eq!(loaded_process.command, "echo");
    assert_eq!(loaded_process.args, vec!["hello"]);
}

#[tokio::test]
async fn test_update_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    let mut process_info = ProcessInfo {
        id: None,
        process_id: "update-test".to_string(),
        name: "update-test".to_string(),
        command: "python".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        status: ProcessStatus {
            state: ProcessState::NotStarted,
            pid: None,
            exit_code: None,
            started_at: None,
            stopped_at: None,
            error: None,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
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
async fn test_delete_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    let process_info = ProcessInfo {
        id: None,
        process_id: "delete-test".to_string(),
        name: "delete-test".to_string(),
        command: "ls".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        status: ProcessStatus {
            state: ProcessState::NotStarted,
            pid: None,
            exit_code: None,
            started_at: None,
            stopped_at: None,
            error: None,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
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
async fn test_process_state_reset() {
    let persistence = PersistenceManager::new().await.unwrap();

    // Test that runtime state is not persisted (by design)
    let process_info = ProcessInfo {
        id: None,
        process_id: "state-test".to_string(),
        name: "state-test".to_string(),
        command: "test".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        status: ProcessStatus {
            state: ProcessState::Running,
            pid: Some(12345),
            exit_code: None,
            started_at: Some(chrono::Utc::now()),
            stopped_at: None,
            error: None,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        tags: vec![],
        auto_start: false,
    };

    persistence.save_process(&process_info).await.unwrap();

    // When loading, state is reset to NotStarted
    // This is by design - processes don't persist their runtime state
    let loaded = persistence.load_all_processes().await.unwrap();
    assert!(loaded.contains_key("state-test"));
    assert_eq!(loaded["state-test"].status.state, ProcessState::NotStarted);
}
