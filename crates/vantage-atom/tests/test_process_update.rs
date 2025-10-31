use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;
use vantage_atom::{messages::CreateProcessRequest, process::ProcessManager};

#[tokio::test]
async fn test_update_process_attributes() {
    // セットアップ
    let manager = ProcessManager::new().await;

    // プロセスを作成
    let request = CreateProcessRequest {
        id: "test_update".to_string(),
        command: "echo".to_string(),
        args: vec!["original".to_string()],
        env: HashMap::new(),
        cwd: None,
        auto_start_on_restore: false,
    };

    manager
        .create_process(
            request.id.clone(),
            request.command,
            request.args,
            request.env,
            request.cwd.map(PathBuf::from),
            request.auto_start_on_restore,
        )
        .await
        .unwrap();

    // プロセス属性を更新
    let mut new_env = HashMap::new();
    new_env.insert("TEST_VAR".to_string(), "test_value".to_string());

    manager
        .update_process(
            "test_update".to_string(),
            Some("ls".to_string()),
            Some(vec!["-la".to_string()]),
            Some(new_env.clone()),
            Some("/tmp".to_string()),
            Some(true),
        )
        .await
        .unwrap();

    // 更新された情報を確認
    let processes = manager.list_processes(None).await;
    let updated_process = processes.iter().find(|p| p.id == "test_update").unwrap();

    // 更新された値を検証
    assert_eq!(updated_process.command, "ls");
    assert_eq!(updated_process.args, vec!["-la"]);
    assert_eq!(updated_process.cwd, Some(PathBuf::from("/tmp")));
    assert!(updated_process.auto_start_on_restore);
    assert_eq!(updated_process.env.get("TEST_VAR").unwrap(), "test_value");
}

#[tokio::test]
async fn test_update_process_persistence() {
    // セットアップ
    let temp_dir = tempdir().unwrap();
    let export_file = temp_dir.path().join("test_export.surql");

    // 最初のマネージャーでプロセスを作成・更新
    {
        let manager = ProcessManager::new().await;

        // プロセスを作成
        manager
            .create_process(
                "persist_test".to_string(),
                "echo".to_string(),
                vec!["hello".to_string()],
                HashMap::new(),
                None,
                false,
            )
            .await
            .unwrap();

        // プロセス属性を更新
        manager
            .update_process(
                "persist_test".to_string(),
                Some("cat".to_string()),
                Some(vec!["file.txt".to_string()]),
                None,
                Some("/home/user".to_string()),
                Some(true),
            )
            .await
            .unwrap();

        // エクスポート
        manager
            .export_processes(Some(export_file.to_str().unwrap().to_string()))
            .await
            .unwrap();
    }

    // 新しいマネージャーでインポート
    {
        let manager = ProcessManager::new().await;

        // インポート
        manager
            .import_processes(export_file.to_str().unwrap())
            .await
            .unwrap();

        // 更新された情報が保持されているか確認
        let processes = manager.list_processes(None).await;
        let restored_process = processes.iter().find(|p| p.id == "persist_test").unwrap();

        // 更新された値が保持されているか検証
        assert_eq!(restored_process.command, "cat");
        assert_eq!(restored_process.args, vec!["file.txt"]);
        assert_eq!(restored_process.cwd, Some(PathBuf::from("/home/user")));
        assert!(restored_process.auto_start_on_restore);
    }
}

#[tokio::test]
async fn test_partial_update() {
    // セットアップ
    let temp_dir = tempdir().unwrap();
    let original_dir = temp_dir.path().join("original");
    std::fs::create_dir(&original_dir).unwrap();

    let manager = ProcessManager::new().await;

    // プロセスを作成
    let mut initial_env = HashMap::new();
    initial_env.insert("INITIAL".to_string(), "value".to_string());

    manager
        .create_process(
            "partial_test".to_string(),
            "echo".to_string(),
            vec!["test".to_string()],
            initial_env,
            Some(original_dir.clone()),
            true,
        )
        .await
        .unwrap();

    // 一部の属性のみ更新（commandとauto_start_on_restoreのみ）
    manager
        .update_process(
            "partial_test".to_string(),
            Some("ls".to_string()), // commandを更新
            None,                   // argsは更新しない
            None,                   // envは更新しない
            None,                   // cwdは更新しない
            Some(false),            // auto_start_on_restoreを更新
        )
        .await
        .unwrap();

    // 更新された情報を確認
    let processes = manager.list_processes(None).await;
    let updated_process = processes.iter().find(|p| p.id == "partial_test").unwrap();

    // 更新された値を検証
    assert_eq!(updated_process.command, "ls");

    // 更新されていない値が保持されているか検証
    assert_eq!(updated_process.args, vec!["test"]);
    assert_eq!(updated_process.cwd, Some(original_dir));
    assert!(!updated_process.auto_start_on_restore); // 更新された値
    assert_eq!(updated_process.env.get("INITIAL").unwrap(), "value");
}
