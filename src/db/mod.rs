use anyhow::{Context, Result};
use std::sync::Arc;
use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// デフォルトのデータディレクトリ
pub const DEFAULT_DATA_DIR: &str = ".ichimi";

pub const DEFAULT_DATA_FILE: &str = "snapshot.surql";

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
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;

        // エクスポート用の構造体を定義（persistence/manager.rsのProcessInfoRecordと同じ）
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct ProcessRecord {
            process_id: String,
            command: String,
            args: Vec<String>,
            env: HashMap<String, String>,
            cwd: Option<String>,
            #[serde(default)]
            auto_start_on_restore: bool,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct EventRecord {
            #[serde(rename = "type")]
            event_type: String,
            process_id: String,
            context: Option<serde_json::Value>,
            metadata: Option<serde_json::Value>,
        }

        info!("Exporting database to: {}", path.display());

        let client = self.client().await;

        // すべてのデータを取得（USE文を含む）
        let query = "USE NS ichimi DB main; SELECT * FROM process; SELECT * FROM process_event; SELECT * FROM template;";
        debug!("Executing export query: {}", query);
        let mut result = client
            .query(query)
            .await
            .context("Failed to fetch data for export")?;

        // SurrealQL形式でエクスポート
        let mut export_content = String::new();
        export_content.push_str("-- Ichimi Server Database Export\n");
        export_content.push_str(&format!("-- Generated at: {}\n\n", chrono::Utc::now()));
        export_content.push_str("USE NS ichimi DB main;\n\n");

        // USE文の結果をスキップ
        let _ = result.take::<Option<()>>(0);

        // プロセスデータのエクスポート（ProcessRecord構造体として直接デシリアライズ）
        let processes: Vec<ProcessRecord> = result.take(1).unwrap_or_default();

        debug!("Found {} processes to export", processes.len());
        for process in processes {
            export_content.push_str(&format!(
                "CREATE process CONTENT {};\n",
                serde_json::to_string(&process)?
            ));
        }

        // イベントデータのエクスポート（EventRecord構造体として直接デシリアライズ）
        let events: Vec<EventRecord> = result.take(2).unwrap_or_default();

        debug!("Found {} events to export", events.len());
        for event in events {
            export_content.push_str(&format!(
                "CREATE process_event CONTENT {};\n",
                serde_json::to_string(&event)?
            ));
        }

        // テンプレートデータのエクスポート
        // ProcessTemplateとして直接デシリアライズ
        let templates: Vec<crate::process::template::ProcessTemplate> = result.take(3).unwrap_or_default();
        
        debug!("Found {} templates to export", templates.len());
        for template in templates {
            export_content.push_str(&format!(
                "CREATE template CONTENT {};\n",
                serde_json::to_string(&template)?
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

        // ネームスペースとデータベースを設定
        client
            .use_ns("ichimi")
            .use_db("main")
            .await
            .context("Failed to set namespace and database")?;

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
        // プロジェクトルート（現在の作業ディレクトリ）の.ichimiディレクトリに保存
        std::path::PathBuf::from(DEFAULT_DATA_DIR).join(DEFAULT_DATA_FILE)
    }

    /// 起動時のデータ復元
    pub async fn restore_on_startup(&self) -> Result<()> {
        let import_path = Self::get_default_data_path();
        if import_path.exists() {
            info!("Restoring data from: {}", import_path.display());
            self.import_from_file(&import_path)
                .await
                .context("Failed to restore data on startup")?;
        } else {
            info!("No existing data file found at: {}", import_path.display());
        }
        Ok(())
    }

    /// 終了時のデータ保存
    pub async fn backup_on_shutdown(&self) -> Result<()> {
        let export_path = Self::get_default_data_path();
        info!("Backing up data to: {}", export_path.display());
        self.export_to_file(&export_path)
            .await
            .context("Failed to backup data on shutdown")?;
        Ok(())
    }
}
