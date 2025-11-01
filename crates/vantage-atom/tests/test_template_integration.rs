/// テンプレート機能の統合テスト
///
/// このテストは実際にSurrealDBに接続し、テンプレートのCRUD操作とプロセス作成を検証します。

use vantage_atom::VantageServer;
use vantage_persistence::db::connection::{DbConnection, DbConfig};
use vantage_persistence::db::schema::SchemaManager;
use vantage_persistence::db::template_repository::{Template, TemplateRepository, TemplateCategory};

/// テスト用のDB接続を作成
async fn setup_test_db() -> DbConnection {
    let config = DbConfig {
        endpoint: "127.0.0.1:30300".to_string(),
        namespace: "vantage".to_string(),
        database: "test".to_string(),
        username: "vtg-local".to_string(),
        password: "vtg-local".to_string(),
    };

    let conn = DbConnection::new(config)
        .await
        .expect("Failed to connect to test database");

    // 既存のtemplateテーブルをクリーンアップ
    let _ = conn.db().query("DELETE FROM template;").await;

    // スキーマを適用
    let schema_manager = SchemaManager::new(conn.db());
    schema_manager
        .apply_all()
        .await
        .expect("Failed to apply schema");

    conn
}

#[tokio::test]
async fn test_template_crud_operations() {
    // テストDB接続
    let db = setup_test_db().await;
    let repo = TemplateRepository::new(db.db());

    // 1. テンプレート作成
    let mut template = Template::new(
        "test-server".to_string(),
        "python -m http.server".to_string(),
    );
    template.description = Some("HTTPサーバーテンプレート".to_string());
    template.category = TemplateCategory::WebServer;
    template.tags = vec!["python".to_string(), "http".to_string()];
    template.args = vec!["8000".to_string()];

    let created = repo.create(template.clone()).await.expect("Failed to create template");
    assert!(created.id.is_some());
    println!("✓ テンプレート作成成功: ID = {:?}", created.id);

    // 2. テンプレート取得（ID）
    let record_id = created.id.as_ref().unwrap();
    let template_id_string = record_id.to_string();
    println!("  RecordId文字列: {}", template_id_string);

    // RecordIdから"template:"プレフィックスを除去してID部分だけを抽出
    let id_part = template_id_string.strip_prefix("template:").unwrap_or(&template_id_string);
    println!("  ID部分: {}", id_part);

    let fetched = repo.get(id_part).await.expect("Failed to get template");
    println!("  取得結果: {:?}", fetched.as_ref().map(|t| &t.name));
    assert!(fetched.is_some());
    assert_eq!(fetched.as_ref().unwrap().name, "test-server");
    println!("✓ テンプレート取得成功（ID）: name = {}", fetched.as_ref().unwrap().name);

    // 3. テンプレート取得（名前）
    let fetched_by_name = repo.get_by_name("test-server").await.expect("Failed to get by name");
    assert!(fetched_by_name.is_some());
    println!("✓ テンプレート取得成功（名前）: name = {}", fetched_by_name.as_ref().unwrap().name);

    // 4. テンプレート一覧取得
    let all_templates = repo.list().await.expect("Failed to list templates");
    assert_eq!(all_templates.len(), 1);
    println!("✓ テンプレート一覧取得成功: {} 件", all_templates.len());

    // 5. テンプレート更新
    let mut updated_template = fetched.unwrap();
    updated_template.description = Some("更新されたHTTPサーバーテンプレート".to_string());
    let updated = repo.update(id_part, updated_template).await.expect("Failed to update template");
    assert_eq!(updated.description, Some("更新されたHTTPサーバーテンプレート".to_string()));
    println!("✓ テンプレート更新成功");

    // 6. 使用回数のインクリメント
    repo.increment_use_count(id_part).await.expect("Failed to increment use count");
    let after_increment = repo.get(id_part).await.expect("Failed to get after increment").unwrap();
    assert_eq!(after_increment.use_count, 1);
    println!("✓ 使用回数インクリメント成功: use_count = {}", after_increment.use_count);

    // 7. カテゴリ検索
    let server_templates = repo.list_by_category(TemplateCategory::WebServer)
        .await
        .expect("Failed to list by category");
    assert_eq!(server_templates.len(), 1);
    println!("✓ カテゴリ検索成功: {} 件", server_templates.len());

    // 8. タグ検索
    let python_templates = repo.search_by_tag("python").await.expect("Failed to search by tag");
    assert_eq!(python_templates.len(), 1);
    println!("✓ タグ検索成功: {} 件", python_templates.len());

    // 9. テンプレート削除
    repo.delete(id_part).await.expect("Failed to delete template");
    let after_delete = repo.get(id_part).await.expect("Failed to get after delete");
    assert!(after_delete.is_none());
    println!("✓ テンプレート削除成功");

    println!("\n✅ 全テストが成功しました");
}

#[tokio::test]
async fn test_vantage_server_initialization() {
    // VantageServerの初期化テスト（DB接続とスキーマ適用）
    let server = VantageServer::new().await.expect("Failed to create VantageServer");

    println!("✓ VantageServer初期化成功（DB接続とスキーマ適用）");

    // シャットダウン
    server.shutdown().await.expect("Failed to shutdown VantageServer");

    println!("✅ VantageServer初期化テストが成功しました");
}
