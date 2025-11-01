//! テンプレートリポジトリ
//!
//! プロセステンプレートのCRUD操作を提供します。
//!
//! # 機能
//!
//! - テンプレートの作成、読み取り、更新、削除
//! - 名前、タグ、カテゴリによる検索
//! - 使用回数の追跡
//!
//! # 使用例
//!
//! ```rust,no_run
//! use vantage_persistence::db::{DbConnection, TemplateRepository};
//! use vantage_persistence::Template;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let conn = DbConnection::new_from_env().await?;
//!     let repo = TemplateRepository::new(conn.db());
//!
//!     // テンプレート作成
//!     let template = Template::new("web-server".to_string(), "npm start".to_string());
//!     let created = repo.create(template).await?;
//!
//!     // 名前で検索
//!     let found = repo.get_by_name("web-server").await?;
//!
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;
use tracing::{debug, info};

/// テンプレートカテゴリ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Database,
    WebServer,
    BuildTool,
    Script,
    Other,
}

impl Default for TemplateCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// プロセステンプレート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub description: Option<String>,
    pub category: TemplateCategory,
    pub tags: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
    pub use_count: i32,
    pub last_used_at: Option<String>,
}

impl Template {
    /// 新しいテンプレートを作成
    pub fn new(name: String, command: String) -> Self {
        Self {
            id: None,
            name,
            description: None,
            category: TemplateCategory::default(),
            tags: Vec::new(),
            command,
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
            created_at: None,
            updated_at: None,
            created_by: None,
            use_count: 0,
            last_used_at: None,
        }
    }

    /// ビルダーパターン
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_category(mut self, category: TemplateCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_cwd(mut self, cwd: String) -> Self {
        self.cwd = Some(cwd);
        self
    }
}

/// テンプレートリポジトリ
pub struct TemplateRepository<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> TemplateRepository<'a> {
    pub fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }

    /// テンプレートを作成
    pub async fn create(&self, template: Template) -> Result<Template> {
        info!("Creating template: {}", template.name);

        let created: Option<Template> = self
            .db
            .create("template")
            .content(template)
            .await
            .context("Failed to create template")?;

        created.context("Template creation returned None")
    }

    /// テンプレートを取得（RecordId使用）
    pub async fn get(&self, id: &str) -> Result<Option<Template>> {
        debug!("Getting template: {}", id);

        // edition="2024"では("table", "id")タプルを直接使用
        let template: Option<Template> = self
            .db
            .select(("template", id))
            .await
            .context("Failed to get template")?;

        Ok(template)
    }

    /// 名前でテンプレートを取得
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Template>> {
        debug!("Getting template by name: {}", name);

        let mut result = self
            .db
            .query("SELECT * FROM template WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await
            .context("Failed to query template by name")?;

        let templates: Vec<Template> = result.take(0).context("Failed to parse query result")?;

        Ok(templates.into_iter().next())
    }

    /// 全テンプレートを取得
    pub async fn list(&self) -> Result<Vec<Template>> {
        debug!("Listing all templates");

        let templates: Vec<Template> = self
            .db
            .select("template")
            .await
            .context("Failed to list templates")?;

        Ok(templates)
    }

    /// カテゴリでフィルタリング
    pub async fn list_by_category(&self, category: TemplateCategory) -> Result<Vec<Template>> {
        debug!("Listing templates by category: {:?}", category);

        let mut result = self
            .db
            .query("SELECT * FROM template WHERE category = $category ORDER BY use_count DESC")
            .bind(("category", category))
            .await
            .context("Failed to query templates by category")?;

        let templates: Vec<Template> = result.take(0).context("Failed to parse query result")?;

        Ok(templates)
    }

    /// タグで検索
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<Template>> {
        debug!("Searching templates by tag: {}", tag);

        let mut result = self
            .db
            .query("SELECT * FROM template WHERE $tag IN tags")
            .bind(("tag", tag.to_string()))
            .await
            .context("Failed to search templates by tag")?;

        let templates: Vec<Template> = result.take(0).context("Failed to parse query result")?;

        Ok(templates)
    }

    /// テンプレートを更新（MERGE使用）
    pub async fn update(&self, id: &str, template: Template) -> Result<Template> {
        info!("Updating template: {}", id);

        let updated: Option<Template> = self
            .db
            .update(("template", id))
            .merge(template)
            .await
            .context("Failed to update template")?;

        updated.context("Template update returned None")
    }

    /// テンプレートを削除
    pub async fn delete(&self, id: &str) -> Result<()> {
        info!("Deleting template: {}", id);

        let _: Option<Template> = self
            .db
            .delete(("template", id))
            .await
            .context("Failed to delete template")?;

        Ok(())
    }

    /// 使用回数を増やす
    pub async fn increment_use_count(&self, id: &str) -> Result<()> {
        debug!("Incrementing use count for template: {}", id);

        let now = chrono::Utc::now().to_rfc3339();
        self.db
            .query("UPDATE type::thing('template', $id) SET use_count += 1, last_used_at = $now")
            .bind(("id", id.to_string()))
            .bind(("now", now))
            .await
            .context("Failed to increment use count")?;

        Ok(())
    }

    /// 人気のテンプレートを取得
    pub async fn get_popular(&self, limit: usize) -> Result<Vec<Template>> {
        debug!("Getting popular templates (limit: {})", limit);

        let mut result = self
            .db
            .query("SELECT * FROM template ORDER BY use_count DESC LIMIT $limit")
            .bind(("limit", limit))
            .await
            .context("Failed to get popular templates")?;

        let templates: Vec<Template> = result.take(0).context("Failed to parse query result")?;

        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbConnection;

    #[tokio::test]
    #[ignore] // SurrealDBサーバーが起動している必要がある
    async fn test_template_crud() {
        let conn = DbConnection::new_default().await.unwrap();
        let repo = TemplateRepository::new(conn.db());

        // Create
        let template = Template::new("test-template".to_string(), "echo".to_string())
            .with_description("Test template".to_string())
            .with_args(vec!["hello".to_string()]);

        let created = repo.create(template).await.unwrap();
        assert_eq!(created.name, "test-template");

        // Get by name
        let fetched = repo.get_by_name("test-template").await.unwrap();
        assert!(fetched.is_some());

        // Get by ID (RecordId経由)
        let id = created.id.as_ref().unwrap();
        let id_str = id.to_string().split(':').nth(1).unwrap();
        let fetched_by_id = repo.get(id_str).await.unwrap();
        assert!(fetched_by_id.is_some());

        // Update
        let mut updated_template = fetched.unwrap();
        updated_template.description = Some("Updated description".to_string());
        let updated = repo.update(id_str, updated_template).await.unwrap();
        assert_eq!(updated.description.unwrap(), "Updated description");

        // Delete
        repo.delete(id_str).await.unwrap();
        let deleted = repo.get(id_str).await.unwrap();
        assert!(deleted.is_none());
    }
}
