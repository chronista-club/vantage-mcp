#[cfg(test)]
mod tests {
    use super::super::template_repository::{Template, TemplateCategory, TemplateRepository};
    use crate::db::connection::{DbConfig, DbConnection};
    use crate::db::schema::SchemaManager;

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

        // スキーマを適用
        let schema_manager = SchemaManager::new(conn.db());
        schema_manager
            .apply_all()
            .await
            .expect("Failed to apply schema");

        conn
    }

    /// テスト用のテンプレートを作成
    fn create_test_template(name: &str) -> Template {
        let mut template = Template::new(name.to_string(), "echo".to_string());
        template.description = Some(format!("Test template: {}", name));
        template.category = TemplateCategory::BuildTool;
        template.args = vec!["hello".to_string()];
        template.tags = vec!["test".to_string()];
        template
    }

    #[tokio::test]
    #[ignore] // SurrealDBサーバーが起動している必要がある
    async fn test_create_template() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        let template = create_test_template("test_create");
        let created = repo.create(template.clone()).await.unwrap();

        assert!(created.id.is_some());
        assert_eq!(created.name, template.name);
        assert_eq!(created.command, template.command);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_template() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // テンプレートを作成
        let template = create_test_template("test_get");
        let created = repo.create(template).await.unwrap();
        let id_string = created.id.as_ref().unwrap().to_string();
        let id = id_string.split(':').nth(1).unwrap();

        // IDで取得
        let retrieved = repo.get(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_get");
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_by_name() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // テンプレートを作成
        let template = create_test_template("test_get_by_name");
        repo.create(template).await.unwrap();

        // 名前で取得
        let retrieved = repo.get_by_name("test_get_by_name").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_get_by_name");
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_template() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // テンプレートを作成
        let template = create_test_template("test_update");
        let created = repo.create(template).await.unwrap();
        let id_string = created.id.as_ref().unwrap().to_string();
        let id = id_string.split(':').nth(1).unwrap();

        // 更新
        let mut updated_template = created.clone();
        updated_template.description = Some("Updated description".to_string());
        updated_template.command = "ls".to_string();

        let updated = repo.update(id, updated_template).await.unwrap();
        assert_eq!(updated.command, "ls");
        assert_eq!(updated.description, Some("Updated description".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_template() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // テンプレートを作成
        let template = create_test_template("test_delete");
        let created = repo.create(template).await.unwrap();
        let id_string = created.id.as_ref().unwrap().to_string();
        let id = id_string.split(':').nth(1).unwrap();

        // 削除
        repo.delete(id).await.unwrap();

        // 削除されたことを確認
        let retrieved = repo.get(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_all_templates() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // 複数のテンプレートを作成
        repo.create(create_test_template("list_test_1"))
            .await
            .unwrap();
        repo.create(create_test_template("list_test_2"))
            .await
            .unwrap();
        repo.create(create_test_template("list_test_3"))
            .await
            .unwrap();

        // 全件取得
        let templates = repo.list().await.unwrap();
        assert!(templates.len() >= 3);
    }

    #[tokio::test]
    #[ignore]
    async fn test_search_by_tag() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // タグ付きテンプレートを作成
        let mut template = create_test_template("tag_test");
        template.tags = vec!["database".to_string(), "postgres".to_string()];
        repo.create(template).await.unwrap();

        // タグで検索
        let results = repo.search_by_tag("database").await.unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.name == "tag_test"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_by_category() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // カテゴリ別テンプレートを作成
        let mut dev_template = create_test_template("dev_test");
        dev_template.category = TemplateCategory::WebServer;
        repo.create(dev_template).await.unwrap();

        let mut monitor_template = create_test_template("monitor_test");
        monitor_template.category = TemplateCategory::Script;
        repo.create(monitor_template).await.unwrap();

        // カテゴリで検索
        let dev_results = repo
            .list_by_category(TemplateCategory::WebServer)
            .await
            .unwrap();
        assert!(!dev_results.is_empty());
        assert!(dev_results.iter().any(|t| t.name == "dev_test"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_unique_name_constraint() {
        let conn = setup_test_db().await;
        let repo = TemplateRepository::new(conn.db());

        // 同じ名前のテンプレートを2つ作成しようとする
        let template1 = create_test_template("unique_test");
        repo.create(template1).await.unwrap();

        let template2 = create_test_template("unique_test");
        let result = repo.create(template2).await;

        // 一意制約違反でエラーになるはず
        assert!(result.is_err());
    }
}
