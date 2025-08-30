/// Unit tests for SurrealDB direct operations
/// 
/// Tests SurrealDB-specific functionality and direct database operations

use ichimi_server::db::Database;
use ichimi_server::persistence::PersistenceManager;
use ichimi_server::process::types::{ProcessInfo, ProcessState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Helper function to create test process
fn create_test_process(id: &str) -> ProcessInfo {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    ProcessInfo {
        id: id.to_string(),
        command: "echo".to_string(),
        args: vec!["hello".to_string(), "world".to_string()],
        env,
        cwd: Some(PathBuf::from("/tmp")),
        state: ProcessState::NotStarted,
        auto_start_on_create: false,
        auto_start_on_restore: true,
    }
}

#[tokio::test]
async fn test_surrealdb_direct_save_and_load() {
    // Test direct SurrealDB operations with shared database instance
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let persistence = PersistenceManager::with_database(database.clone());

    let process = create_test_process("test-direct-save");
    
    // Save and immediately load
    persistence.save_process(&process).await.expect("Failed to save process");
    let loaded = persistence.load_all_processes().await.expect("Failed to load processes");
    
    assert_eq!(loaded.len(), 1, "Should have exactly one process");
    assert!(loaded.contains_key("test-direct-save"), "Process should exist with correct ID");
    
    let loaded_process = &loaded["test-direct-save"];
    assert_eq!(loaded_process.id, "test-direct-save");
    assert_eq!(loaded_process.command, "echo");
    assert_eq!(loaded_process.args, vec!["hello", "world"]);
    assert_eq!(loaded_process.auto_start_on_restore, true);
}

#[tokio::test]
async fn test_surrealdb_multiple_processes() {
    // Test handling multiple processes in SurrealDB
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let persistence = PersistenceManager::with_database(database.clone());

    // Save multiple processes with different configurations
    for i in 1..=5 {
        let mut process = create_test_process(&format!("test-process-{}", i));
        process.command = format!("cmd-{}", i);
        process.auto_start_on_create = i % 2 == 0;  // Even numbers are true
        
        persistence.save_process(&process).await.expect("Failed to save process");
    }
    
    let loaded = persistence.load_all_processes().await.expect("Failed to load processes");
    
    assert_eq!(loaded.len(), 5, "Should have 5 processes");
    
    // Verify each process
    for i in 1..=5 {
        let id = format!("test-process-{}", i);
        assert!(loaded.contains_key(&id), "Process {} should exist", id);
        
        let process = &loaded[&id];
        assert_eq!(process.command, format!("cmd-{}", i));
        assert_eq!(process.auto_start_on_create, i % 2 == 0);
    }
}

#[tokio::test]
async fn test_surrealdb_update_existing() {
    // Test updating existing process (UPDATE semantics in SurrealDB)
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let persistence = PersistenceManager::with_database(database.clone());

    // Save initial process
    let mut process = create_test_process("test-update");
    process.command = "original-command".to_string();
    persistence.save_process(&process).await.expect("Failed to save process");
    
    // Update with same ID
    process.command = "updated-command".to_string();
    process.args = vec!["new-arg".to_string()];
    persistence.save_process(&process).await.expect("Failed to update process");
    
    // Load and verify
    let loaded = persistence.load_all_processes().await.expect("Failed to load processes");
    
    assert_eq!(loaded.len(), 1, "Should still have only one process");
    let loaded_process = &loaded["test-update"];
    assert_eq!(loaded_process.command, "updated-command");
    assert_eq!(loaded_process.args, vec!["new-arg"]);
}

#[tokio::test]
async fn test_surrealdb_query_processes() {
    // Test querying processes with filters
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let persistence = PersistenceManager::with_database(database.clone());

    // Create processes with different commands
    let test_data = vec![
        ("query-1", "python"),
        ("query-2", "node"),
        ("query-3", "python"),
        ("query-4", "ruby"),
    ];

    for (id, cmd) in test_data {
        let mut process = create_test_process(id);
        process.command = cmd.to_string();
        persistence.save_process(&process).await.expect("Failed to save process");
    }

    // Query with filter
    let python_processes = persistence.query_processes("python").await
        .expect("Failed to query processes");
    
    assert_eq!(python_processes.len(), 2, "Should find 2 python processes");
    for process in &python_processes {
        assert_eq!(process.command, "python");
    }
}

#[tokio::test]
async fn test_surrealdb_search_processes() {
    // Test searching processes by command
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let persistence = PersistenceManager::with_database(database.clone());

    // Create processes with searchable content in command
    let mut process1 = create_test_process("search-1");
    process1.command = "grep-pattern".to_string();
    persistence.save_process(&process1).await.expect("Failed to save process");

    let mut process2 = create_test_process("search-2");
    process2.command = "echo-pattern".to_string();
    persistence.save_process(&process2).await.expect("Failed to save process");

    let mut process3 = create_test_process("search-3");
    process3.command = "other-command".to_string();
    persistence.save_process(&process3).await.expect("Failed to save process");

    // Search for "pattern" - query_processes uses command CONTAINS filter
    let results = persistence.query_processes("pattern").await
        .expect("Failed to query processes");
    
    // Both processes with "pattern" in command should be found
    assert_eq!(results.len(), 2, "Should find 2 processes with 'pattern' in command");
    for process in &results {
        assert!(process.command.contains("pattern"), "Command should contain 'pattern'");
    }
}