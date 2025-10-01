use ichimi_server::process::{OutputStream, ProcessFilter, ProcessManager, ProcessStateFilter};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_process_basic_lifecycle() {
    let manager = ProcessManager::new().await;

    // Create a simple echo process
    manager
        .create_process(
            "basic-test".to_string(),
            "echo".to_string(),
            vec!["Hello, Ichimi!".to_string()],
            HashMap::new(),
            None,
            false,
        )
        .await
        .expect("Failed to create process");

    // Start the process
    let pid = manager
        .start_process("basic-test".to_string())
        .await
        .expect("Failed to start process");
    assert!(pid > 0);

    // Give it a moment to execute
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Get output
    let output = manager
        .get_process_output("basic-test".to_string(), OutputStream::Stdout, Some(10))
        .await
        .expect("Failed to get output");

    assert!(!output.is_empty());
    assert!(output[0].contains("Hello, Ichimi!"));

    // Clean up
    manager
        .remove_process("basic-test".to_string())
        .await
        .expect("Failed to remove process");
}

#[tokio::test]
async fn test_process_with_environment() {
    let manager = ProcessManager::new().await;

    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    env.insert("ICHIMI_TEST".to_string(), "running".to_string());

    // Create process with environment variables
    manager
        .create_process(
            "env-test".to_string(),
            "sh".to_string(),
            vec!["-c".to_string(), "echo $TEST_VAR $ICHIMI_TEST".to_string()],
            env,
            None,
            false,
        )
        .await
        .expect("Failed to create process");

    // Start and wait for completion
    let pid = manager
        .start_process("env-test".to_string())
        .await
        .expect("Failed to start process");
    assert!(pid > 0);

    // Wait a bit for output
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Check output contains environment variable values
    let output = manager
        .get_process_output("env-test".to_string(), OutputStream::Stdout, Some(10))
        .await
        .expect("Failed to get output");

    assert!(!output.is_empty());
    assert!(output[0].contains("test_value") || output[0].contains("running"));

    // Clean up
    manager
        .remove_process("env-test".to_string())
        .await
        .expect("Failed to remove process");
}

#[tokio::test]
async fn test_multiple_concurrent_processes() {
    let manager = ProcessManager::new().await;
    let num_processes = 5;

    // Create multiple processes
    for i in 1..=num_processes {
        manager
            .create_process(
                format!("concurrent-{i}"),
                "sh".to_string(),
                vec![
                    "-c".to_string(),
                    format!(
                        "echo 'Process {} started'; sleep 0.5; echo 'Process {} done'",
                        i, i
                    ),
                ],
                HashMap::new(),
                None,
                false,
            )
            .await
            .unwrap_or_else(|_| panic!("Failed to create process {i}"));
    }

    // Start all processes concurrently
    let mut handles = vec![];
    for i in 1..=num_processes {
        let manager_clone = manager.clone();
        let handle =
            tokio::spawn(
                async move { manager_clone.start_process(format!("concurrent-{i}")).await },
            );
        handles.push(handle);
    }

    // Wait for all to start
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok());
    }

    // Give them time to complete
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify all processes have output
    for i in 1..=num_processes {
        let output = manager
            .get_process_output(format!("concurrent-{i}"), OutputStream::Stdout, Some(10))
            .await
            .unwrap_or_else(|_| panic!("Failed to get output for process {i}"));

        assert!(!output.is_empty());
        assert!(
            output
                .iter()
                .any(|line| line.contains(&format!("Process {i}")))
        );
    }

    // Clean up all processes
    for i in 1..=num_processes {
        manager
            .remove_process(format!("concurrent-{i}"))
            .await
            .unwrap_or_else(|_| panic!("Failed to remove process {i}"));
    }
}

#[tokio::test]
async fn test_process_filtering() {
    let manager = ProcessManager::new().await;

    // Create processes with different states
    let test_processes = vec![
        ("filter-running", "sleep", vec!["10"]),
        ("filter-echo", "echo", vec!["test"]),
        ("filter-special", "echo", vec!["special"]),
    ];

    for (id, cmd, args) in &test_processes {
        manager
            .create_process(
                id.to_string(),
                cmd.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
                HashMap::new(),
                None,
                false,
            )
            .await
            .unwrap_or_else(|_| panic!("Failed to create {id}"));
    }

    // Start some processes
    manager
        .start_process("filter-running".to_string())
        .await
        .expect("Failed to start filter-running");
    manager
        .start_process("filter-echo".to_string())
        .await
        .expect("Failed to start filter-echo");

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test filtering by state
    let filter = ProcessFilter {
        state: Some(ProcessStateFilter::Running),
        name_pattern: None,
    };
    let running_processes = manager.list_processes(Some(filter)).await;
    assert!(running_processes.iter().any(|p| p.id == "filter-running"));

    // Test filtering by name pattern
    let filter = ProcessFilter {
        state: None,
        name_pattern: Some("special".to_string()),
    };
    let special_processes = manager.list_processes(Some(filter)).await;
    assert_eq!(special_processes.len(), 1);
    assert_eq!(special_processes[0].id, "filter-special");

    // Clean up
    manager
        .stop_process("filter-running".to_string(), Some(1000))
        .await
        .ok();
    for (id, _, _) in test_processes {
        manager.remove_process(id.to_string()).await.ok();
    }
}

#[tokio::test]
async fn test_process_output_buffering() {
    let manager = ProcessManager::new().await;

    // Create a process that outputs many lines
    let script = r#"
        for i in $(seq 1 50); do
            echo "Line $i"
        done
    "#;

    manager
        .create_process(
            "buffer-test".to_string(),
            "sh".to_string(),
            vec!["-c".to_string(), script.to_string()],
            HashMap::new(),
            None,
            false,
        )
        .await
        .expect("Failed to create process");

    // Start the process
    manager
        .start_process("buffer-test".to_string())
        .await
        .expect("Failed to start process");

    // Wait for completion
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Test getting limited number of lines
    let output_10 = manager
        .get_process_output("buffer-test".to_string(), OutputStream::Stdout, Some(10))
        .await
        .expect("Failed to get 10 lines");
    assert!(output_10.len() <= 10);

    // Test getting all lines (up to buffer limit)
    let output_all = manager
        .get_process_output("buffer-test".to_string(), OutputStream::Stdout, None)
        .await
        .expect("Failed to get all lines");
    assert!(output_all.len() >= 10);

    // Clean up
    manager
        .remove_process("buffer-test".to_string())
        .await
        .expect("Failed to remove process");
}

// Test commented out due to missing dependencies (reqwest, rand)
// This test requires additional dev dependencies to run properly
