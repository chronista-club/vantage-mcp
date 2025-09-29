use anyhow::Result;
use chrono::Utc;
use ichimi_persistence::database::{Database, queries::*};
use tempfile::TempDir;

#[tokio::test]
async fn test_database_connection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    
    let db = Database::new(&db_path).await?;
    assert!(db.test_connection().await.is_ok());
    
    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_clipboard_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;
    let pool = db.pool();
    
    // Store a clipboard entry
    let id = ClipboardQueries::store(
        pool,
        "test_key",
        "test content",
        "text",
        Some(r#"{"source": "test"}"#.to_string()),
        None,
    ).await?;
    assert!(id > 0);
    
    // Get the entry
    let entry = ClipboardQueries::get(pool, "test_key").await?;
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert_eq!(entry.key, "test_key");
    assert_eq!(entry.content, "test content");
    assert_eq!(entry.content_type, "text");
    
    // List entries
    let entries = ClipboardQueries::list(pool).await?;
    assert_eq!(entries.len(), 1);
    
    // Update the entry
    let id2 = ClipboardQueries::store(
        pool,
        "test_key",
        "updated content",
        "text",
        None,
        None,
    ).await?;
    assert_eq!(id, id2); // Should be the same ID (upsert)
    
    // Verify update
    let entry = ClipboardQueries::get(pool, "test_key").await?.unwrap();
    assert_eq!(entry.content, "updated content");
    
    // Delete the entry
    let deleted = ClipboardQueries::delete(pool, "test_key").await?;
    assert!(deleted);
    
    // Verify deletion
    let entry = ClipboardQueries::get(pool, "test_key").await?;
    assert!(entry.is_none());
    
    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_clipboard_expiration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;
    let pool = db.pool();
    
    // Store an expired entry
    let past = Utc::now() - chrono::Duration::hours(1);
    ClipboardQueries::store(
        pool,
        "expired_key",
        "expired content",
        "text",
        None,
        Some(past),
    ).await?;
    
    // Store a valid entry
    let future = Utc::now() + chrono::Duration::hours(1);
    ClipboardQueries::store(
        pool,
        "valid_key",
        "valid content",
        "text",
        None,
        Some(future),
    ).await?;
    
    // Try to get expired entry (should not return)
    let entry = ClipboardQueries::get(pool, "expired_key").await?;
    assert!(entry.is_none());
    
    // Get valid entry (should return)
    let entry = ClipboardQueries::get(pool, "valid_key").await?;
    assert!(entry.is_some());
    
    // Cleanup expired entries
    let cleaned = ClipboardQueries::cleanup_expired(pool).await?;
    assert_eq!(cleaned, 1);
    
    // List should only show valid entry
    let entries = ClipboardQueries::list(pool).await?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "valid_key");
    
    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_process_history() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;
    let pool = db.pool();
    
    // Record process start
    let history_id = ProcessHistoryQueries::record_start(
        pool,
        "test_process_1",
        "Test Process",
        "python",
        Some(r#"["-m", "http.server"]"#.to_string()),
        Some(r#"{"PORT": "8000"}"#.to_string()),
        Some("/tmp"),
    ).await?;
    assert!(history_id > 0);
    
    // Record another process
    ProcessHistoryQueries::record_start(
        pool,
        "test_process_2",
        "Another Process",
        "node",
        Some(r#"["server.js"]"#.to_string()),
        None,
        None,
    ).await?;
    
    // Get history for specific process
    let history = ProcessHistoryQueries::get_history(pool, Some("test_process_1"), None).await?;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].process_id, "test_process_1");
    assert!(history[0].stopped_at.is_none());
    
    // Record process stop
    ProcessHistoryQueries::record_stop(
        pool,
        "test_process_1",
        Some(0),
        None,
    ).await?;
    
    // Verify stop was recorded
    let history = ProcessHistoryQueries::get_history(pool, Some("test_process_1"), None).await?;
    assert!(history[0].stopped_at.is_some());
    assert_eq!(history[0].exit_code, Some(0));
    
    // Get all history
    let all_history = ProcessHistoryQueries::get_history(pool, None, Some(10)).await?;
    assert_eq!(all_history.len(), 2);
    
    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_system_events() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;
    let pool = db.pool();
    
    // Record various events
    SystemEventQueries::record(
        pool,
        "server_start",
        "Server started successfully",
        Some(r#"{"version": "0.1.0"}"#.to_string()),
        "info",
    ).await?;
    
    SystemEventQueries::record(
        pool,
        "process_crash",
        "Process crashed unexpectedly",
        Some(r#"{"process_id": "test_proc"}"#.to_string()),
        "error",
    ).await?;
    
    SystemEventQueries::record(
        pool,
        "config_change",
        "Configuration updated",
        None,
        "warning",
    ).await?;
    
    // Get all events
    let events = SystemEventQueries::get_events(pool, None, None, None).await?;
    assert_eq!(events.len(), 3);
    
    // Filter by event type
    let events = SystemEventQueries::get_events(pool, Some("server_start"), None, None).await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "server_start");
    
    // Filter by severity
    let events = SystemEventQueries::get_events(pool, None, Some("error"), None).await?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, "error");
    
    // Limit results
    let events = SystemEventQueries::get_events(pool, None, None, Some(2)).await?;
    assert_eq!(events.len(), 2);
    
    db.close().await?;
    Ok(())
}
