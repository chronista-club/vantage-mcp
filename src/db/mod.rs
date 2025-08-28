use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct Database {
    client: Arc<RwLock<Surreal<Db>>>,
    db_path: PathBuf,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        info!("Initializing SurrealDB at: {}", db_path.display());

        std::fs::create_dir_all(&db_path.parent().unwrap())
            .context("Failed to create .ichimi directory")?;

        let db = Surreal::new::<RocksDb>(db_path.to_str().unwrap())
            .await
            .context("Failed to create SurrealDB instance")?;

        db.use_ns("ichimi")
            .use_db("main")
            .await
            .context("Failed to set namespace and database")?;

        let database = Self {
            client: Arc::new(RwLock::new(db)),
            db_path,
        };

        database.init_schema().await?;

        info!("SurrealDB initialized successfully");
        Ok(database)
    }

    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".ichimi").join("database.db"))
    }

    pub async fn client(&self) -> tokio::sync::RwLockReadGuard<'_, Surreal<Db>> {
        self.client.read().await
    }

    pub async fn client_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, Surreal<Db>> {
        self.client.write().await
    }

    async fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        let queries = vec![
            // プロセス起動パターン
            r#"
            DEFINE TABLE process_pattern SCHEMAFULL;
            DEFINE FIELD process_id ON process_pattern TYPE string;
            DEFINE FIELD next_processes ON process_pattern TYPE array;
            DEFINE FIELD confidence ON process_pattern TYPE float DEFAULT 0.0;
            DEFINE FIELD context ON process_pattern TYPE object;
            DEFINE INDEX idx_pattern ON process_pattern COLUMNS process_id;
            "#,
            // 時間帯パターン
            r#"
            DEFINE TABLE time_pattern SCHEMAFULL;
            DEFINE FIELD hour_range ON time_pattern TYPE object;
            DEFINE FIELD day_of_week ON time_pattern TYPE array;
            DEFINE FIELD processes ON time_pattern TYPE array;
            DEFINE FIELD frequency ON time_pattern TYPE number DEFAULT 0;
            "#,
            // プロセスイベント
            r#"
            DEFINE TABLE process_event SCHEMAFULL;
            DEFINE FIELD type ON process_event TYPE string;
            DEFINE FIELD process_id ON process_event TYPE string;
            DEFINE FIELD timestamp ON process_event TYPE datetime DEFAULT time::now();
            DEFINE FIELD context ON process_event TYPE object;
            DEFINE FIELD metadata ON process_event TYPE object;
            "#,
            // プロセス定義（グラフ用）
            r#"
            DEFINE TABLE process SCHEMAFULL;
            DEFINE FIELD name ON process TYPE string;
            DEFINE FIELD command ON process TYPE string;
            DEFINE FIELD args ON process TYPE array;
            DEFINE FIELD env ON process TYPE object;
            DEFINE FIELD cwd ON process TYPE string;
            "#,
            // 依存関係
            r#"
            DEFINE TABLE depends_on SCHEMAFULL;
            DEFINE FIELD created_at ON depends_on TYPE datetime DEFAULT time::now();
            "#,
        ];

        let client = self.client().await;
        for query in queries {
            client
                .query(query)
                .await
                .context("Failed to execute schema definition")?;
        }

        debug!("Database schema initialized");
        Ok(())
    }

    pub async fn record_event(
        &self,
        event_type: &str,
        process_id: &str,
        context: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let client = self.client().await;

        let query = r#"
            CREATE process_event CONTENT {
                type: $type,
                process_id: $process_id,
                context: $context,
                metadata: $metadata
            }
        "#;

        client
            .query(query)
            .bind(("type", event_type.to_string()))
            .bind(("process_id", process_id.to_string()))
            .bind(("context", context.unwrap_or(serde_json::Value::Null)))
            .bind(("metadata", metadata.unwrap_or(serde_json::Value::Null)))
            .await
            .context("Failed to record event")?;

        Ok(())
    }
}
