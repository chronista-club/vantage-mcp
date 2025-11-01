//! Vantage Persistence Layer
//!
//! このクレートはVantageの永続化層を提供します。
//!
//! # 主な機能
//!
//! - **インメモリストレージ**: プロセス情報、クリップボード、設定の高速アクセス
//! - **YAML永続化**: 人間が読み書きしやすい設定ファイル形式
//! - **SurrealDB統合**: テンプレートデータの構造化ストレージ
//!
//! # モジュール構成
//!
//! - `persistence`: インメモリストレージとYAMLエクスポート/インポート
//! - `db`: SurrealDBベースのデータベース層（接続、スキーマ、リポジトリ）
//! - `types`: 共通の型定義
//!
//! # 使用例
//!
//! ```rust,no_run
//! use vantage_persistence::{PersistenceManager, DbConnection};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // インメモリストレージの初期化
//!     let persistence = PersistenceManager::new().await?;
//!
//!     // データベース接続
//!     let db = DbConnection::new_from_env().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod persistence;
pub mod types;
pub mod db;

// Re-export main types
pub use persistence::manager::PersistenceManager;

// Re-export types for convenience
pub use types::{
    ClipboardItem, ProcessInfo, ProcessState, ProcessStatus, ProcessTemplate, Settings,
    TemplateVariable, generate_id,
};

// Re-export DB types
pub use db::{DbConnection, TemplateRepository, SchemaManager};
pub use db::template_repository::{Template, TemplateCategory};
