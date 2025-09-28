use ichimi_persistence::kdl::{KdlSnapshot, KdlProcess};
use ichimi_persistence::types::{ProcessInfo, ProcessStatus, ProcessState};
use std::collections::HashMap;
use chrono::Utc;

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
    let kdl_string = snapshot.to_kdl_string();

    println!("{}", kdl_string);

    // Verify content
    assert!(kdl_string.contains("// Ichimi Process Snapshot"));
    assert!(kdl_string.contains("process \"test-server\""));
    assert!(kdl_string.contains("name \"Test Server\""));
    assert!(kdl_string.contains("command \"python\""));
    assert!(kdl_string.contains("auto_start #true"));
    assert!(kdl_string.contains("// 環境変数"));
    assert!(kdl_string.contains("var \"PORT\" \"8000\""));
    assert!(kdl_string.contains("// 実行中"));
    assert!(kdl_string.contains("pid 12345"));

    println!("\n✅ All assertions passed!");
}