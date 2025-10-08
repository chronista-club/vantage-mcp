/// ブラウザE2Eテスト
///
/// Webダッシュボードの基本的な動作をブラウザで確認する
/// headless_chromeを使用してページの読み込みと要素の確認を行う
///
/// 注意：このテストはheadless_chromeクレートに依存します
/// Cargo.tomlのdev-dependenciesに以下を追加してください：
/// headless_chrome = "1.0"

#[cfg(feature = "browser-test")]
mod browser_tests {
    use headless_chrome::{Browser, LaunchOptions};
    use ichimi_persistence::PersistenceManager;
    use ichimi_server::process::ProcessManager;
    use ichimi_server::web::api::create_api_routes;
    use ichimi_server::web::server::AppState;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_dashboard_loads() {
        // テスト用サーバーを起動
        let (server_addr, _server_handle) = start_test_server().await;

        // ブラウザを起動
        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .build()
                .unwrap(),
        )
        .expect("Failed to launch browser");

        let tab = browser.new_tab().expect("Failed to create tab");

        // ダッシュボードにアクセス
        let url = format!("http://{}/", server_addr);
        tab.navigate_to(&url)
            .expect("Failed to navigate to dashboard");

        // ページが読み込まれるまで待機
        tab.wait_until_navigated()
            .expect("Failed to wait for navigation");

        tokio::time::sleep(Duration::from_secs(2)).await;

        // タイトルを確認
        let title = tab.get_title().expect("Failed to get title");
        assert!(
            title.contains("Ichimi") || title.contains("Dashboard"),
            "Unexpected page title: {}",
            title
        );

        // ページの内容を取得
        let content = tab.get_content().expect("Failed to get page content");

        // Vue.jsアプリケーションの存在を確認
        assert!(
            content.contains("id=\"app\"") || content.contains("ichimi"),
            "Vue app not found in page content"
        );
    }

    #[tokio::test]
    async fn test_process_list_visible() {
        let (server_addr, _server_handle) = start_test_server().await;

        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .build()
                .unwrap(),
        )
        .expect("Failed to launch browser");

        let tab = browser.new_tab().expect("Failed to create tab");

        // ダッシュボードにアクセス
        let url = format!("http://{}/", server_addr);
        tab.navigate_to(&url)
            .expect("Failed to navigate to dashboard");

        tab.wait_until_navigated()
            .expect("Failed to wait for navigation");

        // Vue.jsアプリが初期化されるまで待機
        tokio::time::sleep(Duration::from_secs(3)).await;

        // プロセス一覧要素を探す
        let elements = tab
            .wait_for_elements("table, .process-list, [data-testid='process-list']")
            .ok();

        // プロセス一覧が表示されていることを確認
        // （空の場合でも、テーブルや一覧要素は存在するはず）
        assert!(elements.is_some(), "Process list element not found");
    }

    #[tokio::test]
    async fn test_api_endpoint_accessible() {
        let (server_addr, _server_handle) = start_test_server().await;

        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .build()
                .unwrap(),
        )
        .expect("Failed to launch browser");

        let tab = browser.new_tab().expect("Failed to create tab");

        // APIエンドポイントに直接アクセス
        let url = format!("http://{}/status", server_addr);
        tab.navigate_to(&url)
            .expect("Failed to navigate to status endpoint");

        tab.wait_until_navigated()
            .expect("Failed to wait for navigation");

        tokio::time::sleep(Duration::from_millis(500)).await;

        // JSONレスポンスを取得
        let content = tab.get_content().expect("Failed to get page content");

        // JSONレスポンスの構造を確認
        assert!(content.contains("status"));
        assert!(content.contains("version"));
        assert!(content.contains("running") || content.contains("\"status\":\"running\""));
    }

    // ヘルパー関数
    async fn start_test_server() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        let process_manager = ProcessManager::new().await;
        let persistence_manager = PersistenceManager::new()
            .await
            .expect("Failed to create persistence manager");
        let app_state = AppState {
            process_manager: Arc::new(process_manager),
            persistence_manager: Arc::new(persistence_manager),
        };

        let app = create_api_routes().with_state(app_state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // サーバーが起動するまで待機
        tokio::time::sleep(Duration::from_millis(200)).await;

        (addr, handle)
    }
}

// browser-testフィーチャーなしでもコンパイルできるようにダミーテストを追加
#[cfg(not(feature = "browser-test"))]
#[test]
fn browser_tests_disabled() {
    println!("Browser tests are disabled. Enable with --features browser-test");
}
