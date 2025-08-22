use serde::{Serialize, Deserialize};
use surrealdb::sql::{Thing, Datetime};
use surrealdb::{Surreal, engine::local::Db};
use std::collections::HashMap;

/// SurrealDBのモデルを表すトレイト
pub trait Model: Sized + Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// テーブル名を返す
    fn table_name() -> &'static str;
    
    /// レコードIDを返す
    fn id(&self) -> String;
    
    /// Thing型のIDを返す
    fn thing_id(&self) -> Thing {
        Thing::from((Self::table_name(), self.id().as_str()))
    }
    
    /// バリデーションを行う
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// データベース操作を提供する構造体
pub struct ModelDb<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> ModelDb<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }
    
    /// レコードを保存する（新規作成）
    pub async fn save<T: Model>(&self, model: &T) -> Result<(), String> {
        model.validate()?;
        
        let json = serde_json::to_string(model)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        // Use SET syntax for SCHEMAFULL tables
        let query = format!(
            "CREATE {}:`{}` SET {}",
            T::table_name(),
            model.id(),
            self.json_to_set_clause(&json)?
        );
        
        self.db.query(query).await
            .map_err(|e| format!("Failed to save: {}", e))?;
        
        Ok(())
    }
    
    /// Convert JSON to SET clause for SurrealQL
    fn json_to_set_clause(&self, json_str: &str) -> Result<String, String> {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        if let serde_json::Value::Object(map) = value {
            let fields: Vec<String> = map.iter()
                .map(|(key, val)| {
                    let val_str = match (key.as_str(), val) {
                        // Datetimeフィールドは<datetime>でラップ
                        ("created_at" | "updated_at", serde_json::Value::String(s)) => {
                            format!("<datetime>'{}'", s.replace("'", "\\'"))
                        }
                        (_, serde_json::Value::String(s)) => format!("'{}'", s.replace("'", "\\'")),
                        (_, serde_json::Value::Null) => "null".to_string(),
                        _ => serde_json::to_string(val).unwrap_or_else(|_| "null".to_string()),
                    };
                    format!("{} = {}", key, val_str)
                })
                .collect();
            Ok(fields.join(", "))
        } else {
            Err("Expected JSON object".to_string())
        }
    }
    
    /// レコードを更新する
    pub async fn update<T: Model>(&self, model: &T) -> Result<(), String> {
        model.validate()?;
        
        let json = serde_json::to_string(model)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        let query = format!(
            "UPDATE {}:`{}` CONTENT {}",
            T::table_name(),
            model.id(),
            json
        );
        
        self.db.query(query)
            .await
            .map_err(|e| format!("Failed to update: {}", e))?;
        
        Ok(())
    }
    
    /// レコードを削除する
    pub async fn delete<T: Model>(&self, id: &str) -> Result<(), String> {
        let query = format!(
            "DELETE {}:`{}`",
            T::table_name(),
            id
        );
        
        self.db.query(query)
            .await
            .map_err(|e| format!("Failed to delete: {}", e))?;
        
        Ok(())
    }
    
    /// IDでレコードを取得する
    pub async fn find_by_id<T: Model>(&self, id: &str) -> Result<Option<T>, String> {
        let query = format!(
            "SELECT * FROM {}:`{}`",
            T::table_name(),
            id
        );
        
        let mut response = self.db.query(query)
            .await
            .map_err(|e| format!("Failed to find by id: {}", e))?;
        
        let records: Vec<T> = response.take(0)
            .map_err(|e| format!("Failed to extract record: {}", e))?;
        
        Ok(records.into_iter().next())
    }
    
    /// すべてのレコードを取得する
    pub async fn find_all<T: Model>(&self) -> Result<Vec<T>, String> {
        let query = format!("SELECT * FROM {}", T::table_name());
        
        let mut response = self.db.query(query)
            .await
            .map_err(|e| format!("Failed to find all: {}", e))?;
        
        let records: Vec<T> = response.take(0)
            .map_err(|e| format!("Failed to extract records: {}", e))?;
        
        Ok(records)
    }
    
    /// 条件に基づいてレコードを検索する
    pub async fn find_by<T: Model>(&self, filter: &str) -> Result<Vec<T>, String> {
        let query = format!(
            "SELECT * FROM {} WHERE {}",
            T::table_name(),
            filter
        );
        
        let mut response = self.db.query(query)
            .await
            .map_err(|e| format!("Failed to find by filter: {}", e))?;
        
        let records: Vec<T> = response.take(0)
            .map_err(|e| format!("Failed to extract records: {}", e))?;
        
        Ok(records)
    }
    
    /// カウントを取得する
    pub async fn count<T: Model>(&self) -> Result<usize, String> {
        let query = format!("SELECT count() AS total FROM {} GROUP ALL", T::table_name());
        
        let mut response = self.db.query(query)
            .await
            .map_err(|e| format!("Failed to count: {}", e))?;
        
        let result: Vec<serde_json::Value> = response.take(0)
            .map_err(|e| format!("Failed to extract count: {}", e))?;
        
        if let Some(row) = result.first() {
            if let Some(total) = row.get("total").and_then(|v| v.as_u64()) {
                return Ok(total as usize);
            }
        }
        
        Ok(0)
    }
}

/// プロセステーブルのモデル
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Process {
    #[serde(skip_serializing, default = "Process::default_id")]
    pub id: Thing,
    pub process_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub state: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl Process {
    /// デフォルトのIDを生成
    fn default_id() -> Thing {
        Thing::from(("process", ""))
    }
    
    /// 新しいProcessインスタンスを作成
    pub fn new(
        process_id: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<String>,
        state: String,
    ) -> Self {
        let now = Datetime::from(chrono::Utc::now());
        Self {
            id: Thing::from(("process", process_id.as_str())),
            process_id: process_id.clone(),
            command,
            args,
            env,
            cwd,
            state,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// ProcessInfoから変換
    pub fn from_process_info(info: &crate::process::types::ProcessInfo) -> Self {
        Self::new(
            info.id.clone(),
            info.command.clone(),
            info.args.clone(),
            info.env.clone(),
            info.cwd.as_ref().map(|p| p.to_string_lossy().to_string()),
            serde_json::to_string(&info.state).unwrap_or_else(|_| "NotStarted".to_string()),
        )
    }
    
    /// ProcessInfoに変換
    pub fn to_process_info(&self) -> crate::process::types::ProcessInfo {
        use std::path::PathBuf;
        crate::process::types::ProcessInfo {
            id: self.process_id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cwd: self.cwd.as_ref().map(|s| PathBuf::from(s)),
            state: serde_json::from_str(&self.state)
                .unwrap_or(crate::process::types::ProcessState::NotStarted),
        }
    }
}

impl Model for Process {
    fn table_name() -> &'static str {
        "process"
    }
    
    fn id(&self) -> String {
        self.process_id.clone()
    }
    
    fn validate(&self) -> Result<(), String> {
        if self.process_id.is_empty() {
            return Err("process_id cannot be empty".to_string());
        }
        if self.command.is_empty() {
            return Err("command cannot be empty".to_string());
        }
        Ok(())
    }
}

/// スキーマ定義を管理する構造体
pub struct Schema;

impl Schema {
    /// プロセステーブルのスキーマを定義する
    pub async fn define_process_table(db: &Surreal<Db>) -> Result<(), String> {
        // SCHEMALESSを使用して柔軟性を保つ
        let query = r#"
            DEFINE TABLE process SCHEMALESS;
            DEFINE INDEX idx_process_id ON TABLE process COLUMNS process_id UNIQUE;
        "#;
        
        db.query(query)
            .await
            .map_err(|e| format!("Failed to define process table: {}", e))?;
        
        Ok(())
    }
}
