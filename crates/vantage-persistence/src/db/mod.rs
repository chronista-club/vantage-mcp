//! データベース層
//!
//! SurrealDBを使用したテンプレートデータの永続化を提供します。
//!
//! # モジュール
//!
//! - `connection`: データベース接続の管理
//! - `schema`: スキーマ定義の適用と管理
//! - `template_repository`: テンプレートのCRUD操作
//!
//! # 使用例
//!
//! ```rust,no_run
//! use vantage_persistence::db::{DbConnection, SchemaManager, TemplateRepository};
//! use vantage_persistence::Template;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 接続
//!     let conn = DbConnection::new_from_env().await?;
//!
//!     // スキーマ適用
//!     let schema = SchemaManager::new(conn.db());
//!     schema.apply_all().await?;
//!
//!     // リポジトリ経由でCRUD操作
//!     let repo = TemplateRepository::new(conn.db());
//!     let template = Template::new("web-server".to_string(), "npm start".to_string());
//!     let created = repo.create(template).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod connection;
pub mod template_repository;
pub mod schema;

#[cfg(test)]
mod template_repository_tests;

pub use connection::DbConnection;
pub use template_repository::TemplateRepository;
pub use schema::SchemaManager;
