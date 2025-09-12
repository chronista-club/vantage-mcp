use ichimi_persistence::{Database, PersistenceManager, ProcessInfo, ProcessState, ProcessStatus};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// テスト用のプロセス情報を作成
fn create_test_process(id: &str, command: &str) -> ProcessInfo {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), format!("value_{}", id));
    env.insert("ANOTHER_VAR".to_string(), "test".to_string());

    ProcessInfo {
        id: None,
        process_id: id.to_string(),
        name: id.to_string(),
        command: command.to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
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
    }
}

#[tokio::test]
async fn test_export_and_import() {
    // テンポラリディレクトリを作成
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export_import_test.surql");

    // 1. 最初のデータベースインスタンスを作成してデータを追加
    let processes_to_save = {
        let database1 = Arc::new(Database::new().await.expect("Failed to create database"));
        let persistence1 = PersistenceManager::with_database(database1.clone());

        // テストデータを追加
        println!("Phase 1: Adding test data...");
        let mut saved_processes = Vec::new();

        for i in 1..=5 {
            let process =
                create_test_process(&format!("test-process-{}", i), &format!("command-{}", i));
            saved_processes.push(process.clone());

            println!("  Saving process: {}", process.process_id);
            persistence1
                .save_process(&process)
                .await
                .expect("Failed to save process");
        }

        // 保存したデータを確認
        let loaded = persistence1
            .load_all_processes()
            .await
            .expect("Failed to load processes");
        println!("  Loaded {} processes after save", loaded.len());
        assert_eq!(loaded.len(), 5, "Should have saved 5 processes");

        // エクスポート
        println!("\nPhase 2: Exporting to file...");
        database1
            .export_to_file(&export_path)
            .await
            .expect("Failed to export");

        // エクスポートファイルの存在と内容を確認
        assert!(export_path.exists(), "Export file should exist");
        let export_content = std::fs::read_to_string(&export_path).unwrap();
        println!("  Export file size: {} bytes", export_content.len());

        // エクスポート内容の基本的な検証
        assert!(
            export_content.contains("USE NS ichimi DB main"),
            "Should contain USE statement"
        );
        assert!(
            export_content.contains("CREATE process"),
            "Should contain CREATE statements"
        );
        for i in 1..=5 {
            assert!(
                export_content.contains(&format!("test-process-{}", i)),
                "Should contain process ID test-process-{}",
                i
            );
            assert!(
                export_content.contains(&format!("command-{}", i)),
                "Should contain command-{}",
                i
            );
        }

        saved_processes
    };

    // 2. 新しいデータベースインスタンスを作成してインポート
    {
        let database2 = Arc::new(
            Database::new()
                .await
                .expect("Failed to create second database"),
        );
        let persistence2 = PersistenceManager::with_database(database2.clone());

        // インポート前の確認（空であるべき）
        println!("\nPhase 3: Checking new database before import...");
        let before_import = persistence2
            .load_all_processes()
            .await
            .expect("Failed to load processes");
        println!("  Processes before import: {}", before_import.len());
        assert_eq!(before_import.len(), 0, "New database should be empty");

        // インポート
        println!("\nPhase 4: Importing from file...");
        database2
            .import_from_file(&export_path)
            .await
            .expect("Failed to import");

        // インポート後の確認
        println!("\nPhase 5: Verifying imported data...");
        let after_import = persistence2
            .load_all_processes()
            .await
            .expect("Failed to load processes after import");
        println!("  Processes after import: {}", after_import.len());
        assert_eq!(after_import.len(), 5, "Should have imported 5 processes");

        // 各プロセスの詳細を検証
        for original_process in &processes_to_save {
            let imported = after_import.get(&original_process.process_id).expect(&format!(
                "Process {} should exist after import",
                original_process.process_id
            ));

            println!("  Verifying process: {}", original_process.process_id);
            assert_eq!(
                imported.command, original_process.command,
                "Command should match"
            );
            assert_eq!(imported.args, original_process.args, "Args should match");
            assert_eq!(
                imported.env, original_process.env,
                "Environment variables should match"
            );
            assert_eq!(
                imported.cwd, original_process.cwd,
                "Working directory should match"
            );
            assert_eq!(
                imported.auto_start, original_process.auto_start,
                "auto_start should match"
            );
        }

        println!("\n✅ All processes successfully exported and imported!");
    }

    // 3. エクスポートファイルの内容を詳細に確認
    println!("\nPhase 6: Examining export file content...");
    let export_content = std::fs::read_to_string(&export_path).unwrap();
    let lines: Vec<&str> = export_content.lines().collect();

    // ヘッダーの確認
    assert!(
        lines[0].starts_with("-- Ichimi Server Database Export"),
        "Should have export header"
    );
    assert!(
        lines[1].starts_with("-- Generated at:"),
        "Should have generation timestamp"
    );

    // CREATE文の数を確認
    let create_count = lines
        .iter()
        .filter(|line| line.starts_with("CREATE process"))
        .count();
    println!("  Found {} CREATE statements", create_count);
    assert_eq!(create_count, 5, "Should have exactly 5 CREATE statements");

    println!("\n✅ Export/Import test completed successfully!");
}

#[tokio::test]
async fn test_import_nonexistent_file() {
    // 存在しないファイルからのインポートを試みる（エラーにならないことを確認）
    let database = Arc::new(Database::new().await.expect("Failed to create database"));
    let nonexistent_path = PathBuf::from("/tmp/nonexistent_file.surql");

    // 存在しないファイルのインポートは成功するが、何もインポートされない
    let result = database.import_from_file(&nonexistent_path).await;
    assert!(
        result.is_ok(),
        "Import of nonexistent file should not error"
    );

    let persistence = PersistenceManager::with_database(database);
    let processes = persistence
        .load_all_processes()
        .await
        .expect("Failed to load processes");
    assert_eq!(processes.len(), 0, "No processes should be imported");
}
