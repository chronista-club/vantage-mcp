use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use vantage::atom::process::ProcessManager;
use vantage::atom::web::api::create_api_routes;
use vantage::atom::web::server::AppState;
use vantage_persistence::PersistenceManager;

// テスト用のCreateProcessRequest（Serialize追加）
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CreateProcessRequest {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub auto_start_on_restore: bool,
}

/// Web APIの統合テスト
/// 実際のHTTPサーバーを起動してエンドポイントをテストする

#[tokio::test]
async fn test_get_status() {
    let app_state = create_test_app_state().await;
    let app = create_api_routes().with_state(app_state);

    let client = reqwest::Client::new();

    // テスト用サーバーを起動
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // サーバーが起動するまで少し待機
    tokio::time::sleep(Duration::from_millis(100)).await;

    // GET /status をテスト
    let response = client
        .get(format!("http://{}/status", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "running");
    assert!(body["version"].is_string());
    assert!(body["process_count"].is_number());
}

#[tokio::test]
async fn test_process_lifecycle_api() {
    let app_state = create_test_app_state().await;
    let app = create_api_routes().with_state(app_state.clone());

    let client = reqwest::Client::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let base_url = format!("http://{}", addr);

    // 1. プロセスを作成
    let create_req = CreateProcessRequest {
        id: "api-test-echo".to_string(),
        command: "echo".to_string(),
        args: vec!["Hello API Test".to_string()],
        env: HashMap::new(),
        cwd: None,
        auto_start_on_restore: false,
    };

    let response = client
        .post(format!("{}/processes", base_url))
        .json(&create_req)
        .send()
        .await
        .unwrap();

    // POSTリクエストは201 Createdを返す
    assert!(
        response.status() == 200 || response.status() == 201,
        "Expected 200 or 201, got {}",
        response.status()
    );

    // 2. プロセス一覧を取得
    let response = client
        .get(format!("{}/processes", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    let processes: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(processes.iter().any(|p| p["id"] == "api-test-echo"));

    // 3. プロセスを起動
    let response = client
        .post(format!("{}/processes/api-test-echo/start", base_url))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // 少し待機
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 4. プロセスのログを取得
    let response = client
        .get(format!(
            "{}/processes/api-test-echo/logs?stream=stdout&lines=10",
            base_url
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // APIは文字列の配列を直接返す
    let lines: Vec<String> = response.json().await.unwrap();
    assert!(!lines.is_empty(), "Expected log lines, but got empty array");
    assert!(
        lines.iter().any(|line| line.contains("Hello API Test")),
        "Expected to find 'Hello API Test' in logs, got: {:?}",
        lines
    );

    // 5. プロセスを削除
    let response = client
        .delete(format!("{}/processes/api-test-echo", base_url))
        .send()
        .await
        .unwrap();

    // DELETEリクエストは204 No Contentを返す
    assert_eq!(response.status(), 204);
}

#[tokio::test]
async fn test_dashboard_endpoint() {
    let app_state = create_test_app_state().await;
    let app = create_api_routes().with_state(app_state);

    let client = reqwest::Client::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // GET /dashboard をテスト
    let response = client
        .get(format!("http://{}/dashboard", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();

    // ダッシュボードデータの構造を検証
    assert!(body["server"].is_object());
    assert!(body["stats"].is_object());
    assert!(body["processes"].is_array());
    assert!(body["recent_events"].is_array());
    assert!(body["system_metrics"].is_object());

    // サーバー情報を検証
    assert_eq!(body["server"]["status"], "running");
    assert!(body["server"]["version"].is_string());

    // 統計情報を検証
    assert!(body["stats"]["total"].is_number());
    assert!(body["stats"]["running"].is_number());
    assert!(body["stats"]["stopped"].is_number());
}

#[tokio::test]
async fn test_process_filtering() {
    let app_state = create_test_app_state().await;
    let app = create_api_routes().with_state(app_state.clone());

    let client = reqwest::Client::new();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let base_url = format!("http://{}", addr);

    // 複数のプロセスを作成
    for i in 1..=3 {
        let create_req = CreateProcessRequest {
            id: format!("filter-test-{}", i),
            command: "sleep".to_string(),
            args: vec!["10".to_string()],
            env: HashMap::new(),
            cwd: None,
            auto_start_on_restore: false,
        };

        client
            .post(format!("{}/processes", base_url))
            .json(&create_req)
            .send()
            .await
            .unwrap();
    }

    // 1つ目だけ起動
    client
        .post(format!("{}/processes/filter-test-1/start", base_url))
        .send()
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 名前パターンでフィルタ
    let response = client
        .get(format!("{}/processes?name_pattern=filter-test", base_url))
        .send()
        .await
        .unwrap();

    let processes: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(processes.len(), 3);

    // 状態でフィルタ（Running）
    let response = client
        .get(format!("{}/processes?state=running", base_url))
        .send()
        .await
        .unwrap();

    let running_processes: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(running_processes.iter().any(|p| p["id"] == "filter-test-1"));

    // クリーンアップ
    for i in 1..=3 {
        client
            .post(format!("{}/processes/filter-test-{}/stop", base_url, i))
            .send()
            .await
            .ok();

        client
            .delete(format!("{}/processes/filter-test-{}", base_url, i))
            .send()
            .await
            .ok();
    }
}

// ヘルパー関数
async fn create_test_app_state() -> AppState {
    let process_manager = ProcessManager::new().await;
    let persistence_manager = PersistenceManager::new()
        .await
        .expect("Failed to create persistence manager");
    AppState {
        process_manager: Arc::new(process_manager),
        persistence_manager: Arc::new(persistence_manager),
    }
}
