use crate::db::Database;
use crate::process::template::{ProcessTemplate, ClipboardItem};
use crate::process::types::{ProcessInfo, ProcessState};
use crate::web::handlers::Settings;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessInfoRecord {
    process_id: String,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<String>,
    #[serde(default)]
    auto_start_on_restore: bool,
}

impl From<&ProcessInfo> for ProcessInfoRecord {
    fn from(info: &ProcessInfo) -> Self {
        Self {
            process_id: info.id.clone(),
            command: info.command.clone(),
            args: info.args.clone(),
            env: info.env.clone(),
            cwd: info.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
            auto_start_on_restore: info.auto_start_on_restore,
        }
    }
}

impl ProcessInfoRecord {
    fn to_process_info(&self) -> ProcessInfo {
        ProcessInfo {
            id: self.process_id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.as_ref().map(PathBuf::from),
            state: ProcessState::NotStarted,
            auto_start_on_restore: self.auto_start_on_restore,
        }
    }
}

/// Persistence manager using SurrealDB for storage
#[derive(Clone)]
pub struct PersistenceManager {
    database: Arc<Database>,
    snapshot_lock: Arc<tokio::sync::RwLock<()>>,
}


impl PersistenceManager {
    /// Create a new persistence manager with SurrealDB storage
    pub async fn new() -> Result<Self> {
        let database = Arc::new(Database::new().await?);
        let snapshot_lock = Arc::new(tokio::sync::RwLock::new(()));
        Ok(Self { database, snapshot_lock })
    }

    /// Create with existing database instance
    pub fn with_database(database: Arc<Database>) -> Self {
        let snapshot_lock = Arc::new(tokio::sync::RwLock::new(()));
        Self { database, snapshot_lock }
    }

    /// Save or update a process in SurrealDB
    pub async fn save_process(&self, process_info: &ProcessInfo) -> Result<()> {
        tracing::info!(
            "Attempting to save process {} to SurrealDB",
            process_info.id
        );
        let record = ProcessInfoRecord::from(process_info);
        let client = self.database.client().await;

        // DELETEとCREATEを別々のクエリとして実行
        // まず既存のレコードを削除
        let delete_query = "DELETE process WHERE process_id = $process_id";
        let _ = client
            .query(delete_query)
            .bind(("process_id", record.process_id.clone()))
            .await
            .context("Failed to delete existing process")?;

        // 新しいレコードを作成
        let create_query = r#"
            CREATE process CONTENT {
                process_id: $process_id,
                command: $command,
                args: $args,
                env: $env,
                cwd: $cwd,
                auto_start_on_restore: $auto_start_on_restore,
                updated_at: time::now()
            }
        "#;

        let mut create_result = client
            .query(create_query)
            .bind(("process_id", record.process_id.clone()))
            .bind(("command", record.command.clone()))
            .bind(("args", record.args.clone()))
            .bind(("env", record.env.clone()))
            .bind(("cwd", record.cwd.clone()))
            .bind(("auto_start_on_restore", record.auto_start_on_restore))
            .await
            .context("Failed to create process in SurrealDB")?;

        let created: Vec<serde_json::Value> = create_result.take(0).unwrap_or_default();
        tracing::debug!("Created {} records", created.len());

        tracing::debug!("Saved process {} to SurrealDB", process_info.id);

        // Verify the save by immediately querying it back
        let verify_query = "SELECT * FROM process WHERE process_id = $process_id";
        let mut verify_result = client
            .query(verify_query)
            .bind(("process_id", record.process_id.clone()))
            .await
            .context("Failed to verify save")?;

        let verified: Vec<serde_json::Value> = verify_result.take(0).unwrap_or_default();
        tracing::debug!("Verification query returned {} records", verified.len());

        if verified.is_empty() {
            tracing::warn!("Process {} was not found after save", process_info.id);
        }

        Ok(())
    }

    /// Update a process (alias for save_process)
    pub async fn update_process(&self, process_info: &ProcessInfo) -> Result<()> {
        self.save_process(process_info).await
    }

    /// Delete a process from SurrealDB
    pub async fn delete_process(&self, process_id: &str) -> Result<()> {
        let client = self.database.client().await;

        let query = "USE NS ichimi DB main; DELETE process WHERE process_id = $process_id";
        client
            .query(query)
            .bind(("process_id", process_id.to_string()))
            .await
            .context("Failed to delete process from SurrealDB")?;

        tracing::debug!("Deleted process {} from SurrealDB", process_id);
        Ok(())
    }

    /// Load all processes from SurrealDB
    pub async fn load_all_processes(&self) -> Result<HashMap<String, ProcessInfo>, String> {
        let client = self.database.client().await;

        let query = "SELECT * FROM process";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query processes: {e}"))?;

        // ProcessInfoRecord構造体として直接デシリアライズ
        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to deserialize process records: {e}"))?;

        let mut result = HashMap::new();
        for record in records {
            let info = record.to_process_info();
            result.insert(info.id.clone(), info);
        }

        tracing::info!("Loaded {} processes from SurrealDB", result.len());
        Ok(result)
    }

    /// Query processes with simple filters
    pub async fn query_processes(&self, filter: &str) -> Result<Vec<ProcessInfo>, String> {
        let client = self.database.client().await;

        let (query, bind_filter) = if filter.is_empty() {
            ("SELECT * FROM process".to_string(), false)
        } else {
            (
                "SELECT * FROM process WHERE command CONTAINS $filter".to_string(),
                true,
            )
        };

        let mut query_builder = client.query(&query);
        if bind_filter {
            query_builder = query_builder.bind(("filter", filter.to_string()));
        }

        let mut response = query_builder
            .await
            .map_err(|e| format!("Failed to query processes: {e}"))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse process records: {e}"))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
    }

    /// Get process statistics from SurrealDB
    pub async fn get_process_stats(&self) -> Result<serde_json::Value, String> {
        let client = self.database.client().await;

        let query = "SELECT count() as total FROM process GROUP ALL";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to get stats: {e}"))?;

        let stats: Option<serde_json::Value> = response
            .take(0)
            .map_err(|e| format!("Failed to parse stats: {e}"))?;

        Ok(stats.unwrap_or(serde_json::json!({
            "total_processes": 0,
            "running_count": 0,
            "stopped_count": 0,
            "failed_count": 0
        })))
    }

    /// Search processes by command or args
    pub async fn search_processes(&self, search_term: &str) -> Result<Vec<ProcessInfo>, String> {
        let client = self.database.client().await;

        let query = r#"
            SELECT * FROM process 
            WHERE command CONTAINS $term 
               OR array::any(args, |arg| string::lowercase(arg) CONTAINS $term_lower)
        "#;

        let mut response = client
            .query(query)
            .bind(("term", search_term.to_string()))
            .bind(("term_lower", search_term.to_lowercase()))
            .await
            .map_err(|e| format!("Failed to search processes: {e}"))?;

        let records: Vec<ProcessInfoRecord> = response
            .take(0)
            .map_err(|e| format!("Failed to parse search results: {e}"))?;

        Ok(records.into_iter().map(|r| r.to_process_info()).collect())
    }

    /// Export to JSON file (compatibility)
    pub async fn export_to_file(&self, file_path: &str) -> Result<(), String> {
        let processes = self.load_all_processes().await?;
        let json = serde_json::to_string_pretty(&processes)
            .map_err(|e| format!("Failed to serialize processes: {e}"))?;

        std::fs::write(file_path, json).map_err(|e| format!("Failed to write export file: {e}"))?;

        tracing::info!("Exported {} processes to {}", processes.len(), file_path);
        Ok(())
    }

    /// Import from JSON file (compatibility)
    pub async fn import_from_file(&self, file_path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read import file: {e}"))?;

        let imported: HashMap<String, ProcessInfo> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse import file: {e}"))?;

        for (_, info) in imported.iter() {
            self.save_process(info)
                .await
                .map_err(|e| format!("Failed to save imported process: {e}"))?;
        }

        tracing::info!("Imported {} processes from {}", imported.len(), file_path);
        Ok(())
    }

    /// Export to default location
    pub async fn export_default(&self) -> Result<String, String> {
        let path = "ichimi_export.json";
        self.export_to_file(path).await?;
        Ok(path.to_string())
    }

    /// Export all tables to SurrealDB dump format (.surql)
    pub async fn export_to_surql(&self, file_path: &str) -> Result<(), String> {
        // スナップショット中の書き込みを防ぐためにロックを取得
        let _lock = self.snapshot_lock.read().await;
        
        let client = self.database.client().await;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        let mut surql_content = String::new();
        
        // Add header
        surql_content.push_str("-- Ichimi Server Database Export\n");
        surql_content.push_str(&format!("-- Generated at: {} UTC\n\n", chrono::Utc::now()));
        surql_content.push_str("USE NS ichimi DB main;\n\n");
        
        // Export process table
        let processes_query = "USE NS ichimi DB main; SELECT * FROM process";
        let mut response = client
            .query(processes_query)
            .await
            .map_err(|e| format!("Failed to query processes: {}", e))?;
        
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();
        let processes: Vec<serde_json::Value> = response.take(1).unwrap_or_default();
        
        if !processes.is_empty() {
            surql_content.push_str("-- Process table\n");
            for process in &processes {
                let content = serde_json::to_string(process)
                    .map_err(|e| format!("Failed to serialize process: {}", e))?;
                surql_content.push_str(&format!("CREATE process CONTENT {};\n", content));
            }
            surql_content.push_str("\n");
        }
        
        // Export template table
        let templates_query = "USE NS ichimi DB main; SELECT * FROM template";
        let mut response = client
            .query(templates_query)
            .await
            .map_err(|e| format!("Failed to query templates: {}", e))?;
        
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();
        let templates: Vec<serde_json::Value> = response.take(1).unwrap_or_default();
        
        if !templates.is_empty() {
            surql_content.push_str("-- Template table\n");
            for template in &templates {
                let content = serde_json::to_string(template)
                    .map_err(|e| format!("Failed to serialize template: {}", e))?;
                surql_content.push_str(&format!("CREATE template CONTENT {};\n", content));
            }
            surql_content.push_str("\n");
        }
        
        // Export clipboard table
        let clipboard_query = "USE NS ichimi DB main; SELECT * FROM clipboard";
        tracing::debug!("Executing clipboard export query: {}", clipboard_query);
        let mut response = client
            .query(clipboard_query)
            .await
            .map_err(|e| format!("Failed to query clipboard: {}", e))?;
        
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();
        let clipboard_items: Vec<serde_json::Value> = response.take(1).unwrap_or_default();
        tracing::debug!("Found {} clipboard items for export", clipboard_items.len());
        
        if !clipboard_items.is_empty() {
            surql_content.push_str("-- Clipboard table\n");
            for item in &clipboard_items {
                let content = serde_json::to_string(item)
                    .map_err(|e| format!("Failed to serialize clipboard item: {}", e))?;
                surql_content.push_str(&format!("CREATE clipboard CONTENT {};\n", content));
            }
            surql_content.push_str("\n");
        }
        
        // Write to file
        std::fs::write(file_path, surql_content)
            .map_err(|e| format!("Failed to write export file: {}", e))?;
        
        tracing::info!(
            "Exported {} processes, {} templates, {} clipboard items to {}",
            processes.len(),
            templates.len(),
            clipboard_items.len(),
            file_path
        );
        
        Ok(())
    }
    
    /// Import all tables from SurrealDB dump format (.surql)
    pub async fn import_from_surql(&self, file_path: &str) -> Result<(), String> {
        let surql_content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;
        
        let client = self.database.client().await;
        
        // Execute the entire script
        client
            .query(&surql_content)
            .await
            .map_err(|e| format!("Failed to import data: {}", e))?;
        
        tracing::info!("Successfully imported data from {}", file_path);
        Ok(())
    }
    
    /// Create a snapshot of the entire database
    pub async fn create_snapshot(&self) -> Result<String, String> {
        let snapshot_dir = std::env::var("HOME")
            .map(|home| format!("{}/.ichimi", home))
            .unwrap_or_else(|_| ".ichimi".to_string());
        
        let snapshot_path = format!("{}/snapshot.surql", snapshot_dir);
        self.export_to_surql(&snapshot_path).await?;
        Ok(snapshot_path)
    }
    
    /// Restore from the latest snapshot
    pub async fn restore_snapshot(&self) -> Result<(), String> {
        let snapshot_dir = std::env::var("HOME")
            .map(|home| format!("{}/.ichimi", home))
            .unwrap_or_else(|_| ".ichimi".to_string());
        
        let snapshot_path = format!("{}/snapshot.surql", snapshot_dir);
        
        if !std::path::Path::new(&snapshot_path).exists() {
            return Err(format!("Snapshot file not found: {}", snapshot_path));
        }
        
        self.import_from_surql(&snapshot_path).await
    }

    // Settings management
    pub async fn get_settings(&self) -> Result<Settings, String> {
        let client = self.database.client().await;

        // settingsテーブルからdefaultレコードを取得
        let result: Result<Option<Settings>, _> = client.select(("settings", "default")).await;

        match result {
            Ok(Some(settings)) => Ok(settings),
            Ok(None) => Ok(Settings::default()),
            Err(e) => {
                tracing::warn!("Failed to get settings: {}, using defaults", e);
                Ok(Settings::default())
            }
        }
    }

    pub async fn save_settings(&self, settings: Settings) -> Result<(), String> {
        let client = self.database.client().await;

        // settingsテーブルにdefaultレコードを保存/更新
        let result: Result<Option<Settings>, _> = client
            .update(("settings", "default"))
            .content(settings)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to save settings: {}", e)),
        }
    }

    // テンプレート管理メソッド

    /// テンプレートを保存
    pub async fn save_template(&self, template: &ProcessTemplate) -> Result<(), String> {
        let client = self.database.client().await;

        let query = r#"
            USE NS ichimi DB main;
            DELETE template WHERE template_id = $template_id;
            CREATE template CONTENT {
                template_id: $template_id,
                name: $name,
                description: $description,
                category: $category,
                command: $command,
                args: $args,
                env: $env,
                default_cwd: $default_cwd,
                default_auto_start: $default_auto_start,
                variables: $variables,
                tags: $tags,
                created_at: $created_at,
                updated_at: time::now()
            };
        "#;

        client
            .query(query)
            .bind(("template_id", template.template_id.clone()))
            .bind(("name", template.name.clone()))
            .bind(("description", template.description.clone()))
            .bind(("category", template.category.clone()))
            .bind(("command", template.command.clone()))
            .bind(("args", template.args.clone()))
            .bind(("env", template.env.clone()))
            .bind(("default_cwd", template.default_cwd.clone()))
            .bind(("default_auto_start", template.default_auto_start))
            .bind((
                "variables",
                serde_json::to_value(&template.variables).unwrap(),
            ))
            .bind(("tags", template.tags.clone()))
            .bind(("created_at", template.created_at.to_rfc3339()))
            .await
            .map_err(|e| format!("Failed to save template: {}", e))?;

        tracing::debug!("Saved template {} to SurrealDB", template.template_id);
        Ok(())
    }

    /// テンプレートを削除
    pub async fn delete_template(&self, template_id: &str) -> Result<(), String> {
        let client = self.database.client().await;

        let query = "USE NS ichimi DB main; DELETE template WHERE template_id = $template_id";
        client
            .query(query)
            .bind(("template_id", template_id.to_string()))
            .await
            .map_err(|e| format!("Failed to delete template: {}", e))?;

        tracing::debug!("Deleted template {} from SurrealDB", template_id);
        Ok(())
    }

    /// すべてのテンプレートを取得
    pub async fn load_all_templates(&self) -> Result<Vec<ProcessTemplate>, String> {
        let client = self.database.client().await;

        let query = "USE NS ichimi DB main; SELECT * FROM template";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query templates: {e}"))?;

        // USE文の結果をスキップ
        let _ = response.take::<Option<()>>(0);

        // テンプレートレコードを直接ProcessTemplateとして取得
        let templates: Vec<ProcessTemplate> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize template records: {e}"))?;

        tracing::info!("Loaded {} templates from SurrealDB", templates.len());
        Ok(templates)
    }

    /// テンプレートIDで取得
    pub async fn get_template(&self, template_id: &str) -> Result<Option<ProcessTemplate>, String> {
        let client = self.database.client().await;

        let query =
            "USE NS ichimi DB main; SELECT * FROM template WHERE template_id = $template_id";
        let mut response = client
            .query(query)
            .bind(("template_id", template_id.to_string()))
            .await
            .map_err(|e| format!("Failed to query template: {e}"))?;

        // USE文の結果をスキップ
        let _ = response.take::<Option<()>>(0);

        let templates: Vec<ProcessTemplate> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize template record: {e}"))?;

        Ok(templates.first().cloned())
    }

    /// カテゴリでテンプレートを検索
    pub async fn search_templates(
        &self,
        category: Option<String>,
        tags: Vec<String>,
    ) -> Result<Vec<ProcessTemplate>, String> {
        let client = self.database.client().await;

        let mut query = "USE NS ichimi DB main; SELECT * FROM template".to_string();
        let mut conditions = Vec::new();

        if category.is_some() {
            conditions.push("category = $category");
        }

        if !tags.is_empty() {
            conditions.push("array::any(tags, |tag| array::includes($search_tags, tag))");
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut query_builder = client.query(&query);

        if let Some(cat) = category {
            query_builder = query_builder.bind(("category", cat));
        }

        if !tags.is_empty() {
            query_builder = query_builder.bind(("search_tags", tags));
        }

        let mut response = query_builder
            .await
            .map_err(|e| format!("Failed to search templates: {e}"))?;

        // USE文の結果をスキップ
        let _ = response.take::<Option<()>>(0);

        let templates: Vec<ProcessTemplate> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize template records: {e}"))?;

        Ok(templates)
    }

    // クリップボード管理メソッド

    /// クリップボードアイテムを保存
    pub async fn save_clipboard_item(&self, item: &ClipboardItem) -> Result<(), String> {
        // 書き込み操作中はスナップショットをブロック
        let _lock = self.snapshot_lock.write().await;
        
        let client = self.database.client().await;

        let query = r#"
            USE NS ichimi DB main;
            CREATE type::thing('clipboard', $clipboard_id) CONTENT {
                clipboard_id: $clipboard_id,
                content: $content,
                filename: $filename,
                created_at: $created_at,
                updated_at: $updated_at,
                content_type: $content_type,
                tags: $tags
            };
        "#;

        client
            .query(query)
            .bind(("clipboard_id", item.id.clone()))
            .bind(("content", item.content.clone()))
            .bind(("filename", item.filename.clone()))
            .bind(("created_at", item.created_at.to_rfc3339()))
            .bind(("updated_at", item.updated_at.to_rfc3339()))
            .bind(("content_type", item.content_type.clone()))
            .bind(("tags", item.tags.clone()))
            .await
            .map_err(|e| format!("Failed to save clipboard item: {}", e))?;

        tracing::debug!("Saved clipboard item {} to SurrealDB", item.id);
        Ok(())
    }

    /// 最新のクリップボードアイテムを取得（単一アイテム用）
    pub async fn get_latest_clipboard_item(&self) -> Result<Option<ClipboardItem>, String> {
        tracing::debug!("Getting latest clipboard item from database");
        let client = self.database.client().await;

        let query = "USE NS ichimi DB main; SELECT clipboard_id as id, content, filename, created_at, updated_at, content_type, tags FROM clipboard ORDER BY updated_at DESC LIMIT 1";
        tracing::debug!("Executing clipboard query: {}", query);
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query latest clipboard item: {}", e))?;

        // USE文の結果をスキップ
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();

        let items: Vec<ClipboardItem> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize latest clipboard item: {}", e))?;

        tracing::debug!("Query returned {} clipboard items", items.len());
        Ok(items.into_iter().next())
    }

    /// クリップボードにテキストを設定
    pub async fn set_clipboard_text(&self, content: String) -> Result<ClipboardItem, String> {
        let item = ClipboardItem::new(content, None, Some("text".to_string()));
        self.save_clipboard_item(&item).await?;
        Ok(item)
    }

    /// クリップボードにファイルを設定
    pub async fn set_clipboard_file(&self, content: String, filename: String) -> Result<ClipboardItem, String> {
        let item = ClipboardItem::new(content, Some(filename), Some("file".to_string()));
        self.save_clipboard_item(&item).await?;
        Ok(item)
    }

    /// すべてのクリップボードアイテムを取得（履歴表示用）
    pub async fn load_all_clipboard_items(&self) -> Result<Vec<ClipboardItem>, String> {
        let client = self.database.client().await;

        let query = "USE NS ichimi DB main; SELECT * FROM clipboard ORDER BY updated_at DESC";
        let mut response = client
            .query(query)
            .await
            .map_err(|e| format!("Failed to query clipboard items: {}", e))?;

        // USE文の結果をスキップ
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();

        let items: Vec<ClipboardItem> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize clipboard items: {}", e))?;

        tracing::info!("Loaded {} clipboard items from SurrealDB", items.len());
        Ok(items)
    }

    /// Get clipboard history with optional limit
    pub async fn get_clipboard_history(&self, limit: Option<usize>) -> Result<Vec<ClipboardItem>, String> {
        let client = self.database.client().await;

        let query = match limit {
            Some(limit) => format!(
                "USE NS ichimi DB main; SELECT clipboard_id as id, content, filename, created_at, updated_at, content_type, tags FROM clipboard ORDER BY updated_at DESC LIMIT {}", 
                limit
            ),
            None => "USE NS ichimi DB main; SELECT clipboard_id as id, content, filename, created_at, updated_at, content_type, tags FROM clipboard ORDER BY updated_at DESC".to_string(),
        };

        let mut response = client
            .query(&query)
            .await
            .map_err(|e| format!("Failed to query clipboard history: {}", e))?;

        // USE文の結果をスキップ
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();

        let items: Vec<ClipboardItem> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize clipboard history: {}", e))?;

        Ok(items)
    }

    /// Search clipboard items by content
    pub async fn search_clipboard_items(&self, query: &str, limit: Option<usize>) -> Result<Vec<ClipboardItem>, String> {
        let client = self.database.client().await;

        // パラメータ化されたクエリを使用してSQLインジェクションを防ぐ
        let query_string = query.to_string(); // 所有権を持つ文字列に変換
        
        // パラメータ化されたクエリを使用
        let mut response = if let Some(l) = limit {
            client
                .query("USE NS ichimi DB main")
                .query(
                    "SELECT clipboard_id as id, content, filename, created_at, updated_at 
                     FROM clipboard 
                     WHERE content CONTAINS $query OR filename CONTAINS $query 
                     ORDER BY updated_at DESC 
                     LIMIT $limit"
                )
                .bind(("query", query_string.clone()))
                .bind(("limit", l))
                .await
                .map_err(|e| format!("Failed to search clipboard items: {}", e))?
        } else {
            client
                .query("USE NS ichimi DB main")
                .query(
                    "SELECT clipboard_id as id, content, filename, created_at, updated_at 
                     FROM clipboard 
                     WHERE content CONTAINS $query OR filename CONTAINS $query 
                     ORDER BY updated_at DESC"
                )
                .bind(("query", query_string))
                .await
                .map_err(|e| format!("Failed to search clipboard items: {}", e))?
        };

        // USE文の結果をスキップ
        let _: Vec<surrealdb::Value> = response.take(0).unwrap_or_default();

        let items: Vec<ClipboardItem> = response
            .take(1)
            .map_err(|e| format!("Failed to deserialize search results: {}", e))?;

        Ok(items)
    }

    /// Clear all clipboard items
    pub async fn clear_clipboard(&self) -> Result<(), String> {
        // 書き込み操作中はスナップショットをブロック
        let _lock = self.snapshot_lock.write().await;
        
        let client = self.database.client().await;

        client
            .query("USE NS ichimi DB main; DELETE clipboard")
            .await
            .map_err(|e| format!("Failed to clear clipboard: {}", e))?;

        tracing::info!("Cleared all clipboard items from SurrealDB");
        Ok(())
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        // For tests only - this will panic if used
        panic!("Default PersistenceManager should not be used in production")
    }
}
