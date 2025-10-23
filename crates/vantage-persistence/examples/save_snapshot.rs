use chrono::Utc;
use std::collections::HashMap;
use vantage_persistence::{PersistenceManager, ProcessInfo, ProcessState, ProcessStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating sample processes and saving snapshot...\n");

    // Create persistence manager
    let manager = PersistenceManager::new().await?;

    // Create sample processes
    let mut env1 = HashMap::new();
    env1.insert("PORT".to_string(), "8080".to_string());
    env1.insert("NODE_ENV".to_string(), "production".to_string());

    let web_server = ProcessInfo {
        id: None,
        process_id: "web-server-01".to_string(),
        name: "Production Web Server".to_string(),
        command: "node".to_string(),
        args: vec!["server.js".to_string(), "--cluster".to_string()],
        env: env1,
        cwd: Some("/var/www/app".to_string()),
        status: ProcessStatus {
            state: ProcessState::Running,
            pid: Some(12345),
            exit_code: None,
            started_at: Some(Utc::now()),
            stopped_at: None,
            error: None,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec!["production".to_string(), "web".to_string()],
        auto_start_on_restore: true,
    };

    let mut env2 = HashMap::new();
    env2.insert(
        "DATABASE_URL".to_string(),
        "postgres://localhost/myapp".to_string(),
    );
    env2.insert("WORKERS".to_string(), "4".to_string());

    let background_worker = ProcessInfo {
        id: None,
        process_id: "worker-01".to_string(),
        name: "Background Job Worker".to_string(),
        command: "python".to_string(),
        args: vec![
            "worker.py".to_string(),
            "--queue".to_string(),
            "default".to_string(),
        ],
        env: env2,
        cwd: Some("/opt/workers".to_string()),
        status: ProcessStatus {
            state: ProcessState::Running,
            pid: Some(23456),
            exit_code: None,
            started_at: Some(Utc::now()),
            stopped_at: None,
            error: None,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec!["worker".to_string(), "background".to_string()],
        auto_start_on_restore: true,
    };

    let monitoring = ProcessInfo {
        id: None,
        process_id: "monitor-01".to_string(),
        name: "System Monitor".to_string(),
        command: "prometheus".to_string(),
        args: vec![
            "--config.file".to_string(),
            "/etc/prometheus/prometheus.yml".to_string(),
        ],
        env: HashMap::new(),
        cwd: Some("/opt/monitoring".to_string()),
        status: ProcessStatus {
            state: ProcessState::Stopped,
            pid: None,
            exit_code: Some(0),
            started_at: Some(Utc::now() - chrono::Duration::hours(2)),
            stopped_at: Some(Utc::now() - chrono::Duration::minutes(30)),
            error: None,
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
        tags: vec!["monitoring".to_string(), "metrics".to_string()],
        auto_start_on_restore: false,
    };

    // Save processes to manager
    manager.save_process(&web_server).await?;
    manager.save_process(&background_worker).await?;
    manager.save_process(&monitoring).await?;

    println!("Saved 3 sample processes to manager\n");

    // Create snapshot directory
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let snapshot_dir = std::path::Path::new(&home).join(".vantage");
    std::fs::create_dir_all(&snapshot_dir)?;

    // Export snapshot to KDL
    let snapshot_path = snapshot_dir.join("processes.kdl");
    let exported_path = manager
        .export_snapshot(Some(snapshot_path.to_str().unwrap()), false)
        .await?;

    println!("✅ Snapshot exported to: {}", exported_path);

    // Also export auto-start only snapshot
    let auto_start_path = snapshot_dir.join("auto_start.kdl");
    let auto_exported = manager
        .export_snapshot(Some(auto_start_path.to_str().unwrap()), true)
        .await?;

    println!("✅ Auto-start snapshot exported to: {}", auto_exported);

    // Read and display the snapshot
    let content = std::fs::read_to_string(&snapshot_path)?;
    println!("\n📄 Snapshot content:\n");
    println!("{}", content);

    Ok(())
}
