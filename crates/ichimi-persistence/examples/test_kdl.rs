use chrono::Utc;
use ichimi_persistence::kdl_serde::KdlSnapshot;
use ichimi_persistence::types::{ProcessInfo, ProcessState, ProcessStatus};
use std::collections::HashMap;

fn main() {
    println!("Testing KDL generation...\n");

    // Create test process
    let mut env = HashMap::new();
    env.insert("PORT".to_string(), "8000".to_string());
    env.insert("DEBUG".to_string(), "true".to_string());

    let process = ProcessInfo {
        id: None,
        process_id: "test-server".to_string(),
        name: "Test Server".to_string(),
        command: "python".to_string(),
        args: vec!["-m".to_string(), "http.server".to_string()],
        env,
        cwd: Some("/tmp".to_string()),
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
        tags: vec!["web".to_string(), "test".to_string()],
        auto_start_on_restore: true,
    };

    // Generate KDL
    let snapshot = KdlSnapshot::from_processes(vec![process]);
    let kdl_string = snapshot.to_kdl_string().unwrap();

    println!("{}", kdl_string);

    // Verify content
    assert!(kdl_string.contains("// Ichimi Process Snapshot"));
    assert!(kdl_string.contains("process \"test-server\""));
    assert!(kdl_string.contains("name=\"Test Server\""));
    assert!(kdl_string.contains("command=\"python\""));
    assert!(kdl_string.contains("auto_start=true"));
    assert!(kdl_string.contains("env {"));
    assert!(kdl_string.contains("var \"PORT\" \"8000\""));
    assert!(kdl_string.contains("state \"running\""));
    assert!(kdl_string.contains("pid=12345"));

    println!("\n✅ All assertions passed!");
}
