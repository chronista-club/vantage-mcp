use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
};
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Clone)]
pub struct Database {
    client: Arc<RwLock<Surreal<Db>>>,
    db_path: PathBuf,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        info!("Initializing SurrealDB at: {}", db_path.display());

        // dataディレクトリが存在しない場合は作成
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create data directory")?;
        }

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

        info!("SurrealDB initialized successfully");
        Ok(database)
    }

    fn get_db_path() -> Result<PathBuf> {
        // 環境変数からカスタムパスを取得、なければデフォルト
        if let Ok(custom_path) = std::env::var("ICHIMI_DB_PATH") {
            return Ok(PathBuf::from(custom_path));
        }

        // プロジェクトルートのdata/ichimi.dbをデフォルトとする
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        Ok(current_dir.join("data").join("ichimi.db"))
    }

    pub async fn client(&self) -> tokio::sync::RwLockReadGuard<'_, Surreal<Db>> {
        self.client.read().await
    }

    pub async fn client_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, Surreal<Db>> {
        self.client.write().await
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
