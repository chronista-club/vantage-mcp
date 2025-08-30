/// Unit tests for persistence layer (PersistenceManager)
/// 
/// Tests basic CRUD operations for process persistence

use ichimi_server::persistence::PersistenceManager;
use ichimi_server::process::types::{ProcessInfo, ProcessState};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
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
        auto_start_on_create: false,
        auto_start_on_restore: false,
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
async fn test_update_process() {
    let persistence = PersistenceManager::new().await.unwrap();

    let mut process_info = ProcessInfo {
        id: "update-test".to_string(),
        command: "python".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start_on_create: false,
        auto_start_on_restore: false,
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
        id: "delete-test".to_string(),
        command: "ls".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::NotStarted,
        auto_start_on_create: false,
        auto_start_on_restore: false,
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
        id: "state-test".to_string(),
        command: "test".to_string(),
        args: vec![],
        env: HashMap::new(),
        cwd: None,
        state: ProcessState::Running {
            pid: 12345,
            started_at: chrono::Utc::now(),
        },
        auto_start_on_create: false,
        auto_start_on_restore: false,
    };

    persistence.save_process(&process_info).await.unwrap();

    // When loading, state is reset to NotStarted
    // This is by design - processes don't persist their runtime state
    let loaded = persistence.load_all_processes().await.unwrap();
    assert!(loaded.contains_key("state-test"));
    assert_eq!(loaded["state-test"].state, ProcessState::NotStarted);
}