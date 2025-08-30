use anyhow::{Context, Result};
use std::sync::Arc;
use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
};
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    client: Arc<RwLock<Surreal<Db>>>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        info!("Initializing in-memory SurrealDB");

        // 常にインメモリデータベースを使用
        let db = Surreal::new::<Mem>(())
            .await
            .context("Failed to create in-memory SurrealDB instance")?;

        db.use_ns("ichimi")
            .use_db("main")
            .await
            .context("Failed to set namespace and database")?;

        let database = Self {
            client: Arc::new(RwLock::new(db)),
        };

        info!("In-memory SurrealDB initialized successfully");
        Ok(database)
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

    /// データをSurrealQLファイルにエクスポート
    pub async fn export_to_file(&self, path: &std::path::Path) -> Result<()> {
        info!("Exporting database to: {}", path.display());

        let client = self.client().await;

        // すべてのデータを取得
        let mut result = client
            .query("SELECT * FROM process; SELECT * FROM process_event;")
            .await
            .context("Failed to fetch data for export")?;

        // SurrealQL形式でエクスポート
        let mut export_content = String::new();
        export_content.push_str("-- Ichimi Server Database Export\n");
        export_content.push_str(&format!("-- Generated at: {}\n\n", chrono::Utc::now()));

        // プロセスデータのエクスポート
        let processes: Vec<serde_json::Value> = result.take(0)?;
        for process in processes {
            export_content.push_str(&format!(
                "CREATE process CONTENT {};\n",
                serde_json::to_string(&process)?
            ));
        }

        // イベントデータのエクスポート
        let events: Vec<serde_json::Value> = result.take(1)?;
        for event in events {
            export_content.push_str(&format!(
                "CREATE process_event CONTENT {};\n",
                serde_json::to_string(&event)?
            ));
        }

        // ファイルに書き込み
        std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")))?;
        std::fs::write(path, export_content).context("Failed to write export file")?;

        info!("Database exported successfully");
        Ok(())
    }

    /// SurrealQLファイルからデータをインポート
    pub async fn import_from_file(&self, path: &std::path::Path) -> Result<()> {
        if !path.exists() {
            info!("Import file not found: {}", path.display());
            return Ok(());
        }

        info!("Importing database from: {}", path.display());

        let content = std::fs::read_to_string(path).context("Failed to read import file")?;

        let client = self.client().await;

        // SurrealQLを実行
        client
            .query(&content)
            .await
            .context("Failed to execute import queries")?;

        info!("Database imported successfully");
        Ok(())
    }

    /// デフォルトのデータファイルパスを取得
    pub fn get_default_data_path() -> std::path::PathBuf {
        // プロジェクトルートの .ichimi ディレクトリに保存
        std::path::PathBuf::from(".ichimi").join("data.surql")
    }
}
