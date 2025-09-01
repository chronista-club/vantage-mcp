use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Thing;

/// プロセステンプレート - よく使うプロセス設定を保存して再利用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTemplate {
    /// SurrealDB record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Thing>,
    
    /// テンプレートの一意識別子
    pub template_id: String,
    
    /// テンプレートの表示名
    pub name: String,
    
    /// テンプレートの説明
    pub description: Option<String>,
    
    /// カテゴリ（例: "development", "monitoring", "utility"）
    pub category: Option<String>,
    
    /// 実行するコマンド
    pub command: String,
    
    /// コマンド引数のテンプレート（変数置換可能）
    pub args: Vec<String>,
    
    /// 環境変数のテンプレート
    pub env: HashMap<String, String>,
    
    /// デフォルトの作業ディレクトリ
    pub default_cwd: Option<String>,
    
    /// 復元時に自動起動するかのデフォルト値
    pub default_auto_start: bool,
    
    /// テンプレート変数の定義
    pub variables: Vec<TemplateVariable>,
    
    /// テンプレートのタグ
    pub tags: Vec<String>,
    
    /// 作成日時
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// 更新日時
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// テンプレート変数の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// 変数名（例: "PORT", "LOG_LEVEL"）
    pub name: String,
    
    /// 変数の説明
    pub description: Option<String>,
    
    /// 変数の型
    pub var_type: VariableType,
    
    /// デフォルト値
    pub default_value: Option<String>,
    
    /// 必須かどうか
    pub required: bool,
    
    /// 検証ルール
    pub validation: Option<VariableValidation>,
}

/// 変数の型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Path,
    Enum(Vec<String>),
}

/// 変数の検証ルール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValidation {
    /// 最小値（数値型の場合）
    pub min: Option<i64>,
    
    /// 最大値（数値型の場合）
    pub max: Option<i64>,
    
    /// 正規表現パターン（文字列型の場合）
    pub pattern: Option<String>,
    
    /// カスタムエラーメッセージ
    pub error_message: Option<String>,
}

impl ProcessTemplate {
    /// 新しいテンプレートを作成
    pub fn new(template_id: String, name: String, command: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: None,
            template_id,
            name,
            description: None,
            category: None,
            command,
            args: Vec::new(),
            env: HashMap::new(),
            default_cwd: None,
            default_auto_start: false,
            variables: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// テンプレートから ProcessInfo を生成
    pub fn instantiate(
        &self,
        process_id: String,
        values: HashMap<String, String>,
    ) -> Result<crate::process::types::ProcessInfo, String> {
        // 変数の検証
        self.validate_variables(&values)?;
        
        // 変数置換
        let args = self.substitute_variables(&self.args, &values)?;
        let env = self.substitute_env_variables(&self.env, &values)?;
        let cwd = self.default_cwd
            .as_ref()
            .map(|cwd| self.substitute_string(cwd, &values))
            .transpose()?
            .map(std::path::PathBuf::from);
        
        Ok(crate::process::types::ProcessInfo {
            id: process_id,
            command: self.command.clone(),
            args,
            env,
            cwd,
            state: crate::process::types::ProcessState::NotStarted,
            auto_start_on_restore: self.default_auto_start,
        })
    }
    
    /// 変数の検証
    fn validate_variables(&self, values: &HashMap<String, String>) -> Result<(), String> {
        for var in &self.variables {
            if var.required && !values.contains_key(&var.name) {
                return Err(format!("Required variable '{}' is missing", var.name));
            }
            
            if let Some(value) = values.get(&var.name) {
                self.validate_variable_value(var, value)?;
            }
        }
        Ok(())
    }
    
    /// 個別の変数値を検証
    fn validate_variable_value(&self, var: &TemplateVariable, value: &str) -> Result<(), String> {
        match &var.var_type {
            VariableType::Number => {
                let num = value.parse::<i64>()
                    .map_err(|_| format!("Variable '{}' must be a number", var.name))?;
                
                if let Some(validation) = &var.validation {
                    if let Some(min) = validation.min {
                        if num < min {
                            return Err(validation.error_message.clone()
                                .unwrap_or_else(|| format!("Variable '{}' must be >= {}", var.name, min)));
                        }
                    }
                    if let Some(max) = validation.max {
                        if num > max {
                            return Err(validation.error_message.clone()
                                .unwrap_or_else(|| format!("Variable '{}' must be <= {}", var.name, max)));
                        }
                    }
                }
            },
            VariableType::Boolean => {
                value.parse::<bool>()
                    .map_err(|_| format!("Variable '{}' must be a boolean", var.name))?;
            },
            VariableType::Enum(options) => {
                if !options.contains(&value.to_string()) {
                    return Err(format!("Variable '{}' must be one of: {:?}", var.name, options));
                }
            },
            VariableType::String | VariableType::Path => {
                if let Some(validation) = &var.validation {
                    if let Some(pattern) = &validation.pattern {
                        let re = regex::Regex::new(pattern)
                            .map_err(|e| format!("Invalid regex pattern: {}", e))?;
                        if !re.is_match(value) {
                            return Err(validation.error_message.clone()
                                .unwrap_or_else(|| format!("Variable '{}' does not match pattern", var.name)));
                        }
                    }
                }
            },
        }
        Ok(())
    }
    
    /// 文字列リストの変数置換
    fn substitute_variables(
        &self,
        args: &[String],
        values: &HashMap<String, String>,
    ) -> Result<Vec<String>, String> {
        args.iter()
            .map(|arg| self.substitute_string(arg, values))
            .collect()
    }
    
    /// 環境変数の変数置換
    fn substitute_env_variables(
        &self,
        env: &HashMap<String, String>,
        values: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, String> {
        env.iter()
            .map(|(key, val)| {
                Ok((key.clone(), self.substitute_string(val, values)?))
            })
            .collect()
    }
    
    /// 単一文字列の変数置換
    fn substitute_string(
        &self,
        input: &str,
        values: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut result = input.to_string();
        
        // {{VARIABLE}} 形式の変数を置換
        for var in &self.variables {
            let placeholder = format!("{{{{{}}}}}", var.name);
            if result.contains(&placeholder) {
                let value = values.get(&var.name)
                    .or_else(|| var.default_value.as_ref())
                    .ok_or_else(|| format!("No value provided for variable '{}'", var.name))?;
                result = result.replace(&placeholder, value);
            }
        }
        
        Ok(result)
    }
}

/// 事前定義されたテンプレートを提供
pub fn get_builtin_templates() -> Vec<ProcessTemplate> {
    vec![
        // Node.js開発サーバー
        ProcessTemplate {
            id: None,
            template_id: "nodejs-dev".to_string(),
            name: "Node.js Development Server".to_string(),
            description: Some("Run a Node.js development server with hot reload".to_string()),
            category: Some("development".to_string()),
            command: "npm".to_string(),
            args: vec!["run".to_string(), "dev".to_string()],
            env: HashMap::from([
                ("NODE_ENV".to_string(), "development".to_string()),
                ("PORT".to_string(), "{{PORT}}".to_string()),
            ]),
            default_cwd: Some("{{PROJECT_PATH}}".to_string()),
            default_auto_start: false,
            variables: vec![
                TemplateVariable {
                    name: "PORT".to_string(),
                    description: Some("Server port".to_string()),
                    var_type: VariableType::Number,
                    default_value: Some("3000".to_string()),
                    required: false,
                    validation: Some(VariableValidation {
                        min: Some(1024),
                        max: Some(65535),
                        pattern: None,
                        error_message: Some("Port must be between 1024 and 65535".to_string()),
                    }),
                },
                TemplateVariable {
                    name: "PROJECT_PATH".to_string(),
                    description: Some("Path to the project directory".to_string()),
                    var_type: VariableType::Path,
                    default_value: None,
                    required: true,
                    validation: None,
                },
            ],
            tags: vec!["nodejs".to_string(), "development".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        
        // Python仮想環境
        ProcessTemplate {
            id: None,
            template_id: "python-venv".to_string(),
            name: "Python Virtual Environment".to_string(),
            description: Some("Run Python script in a virtual environment".to_string()),
            category: Some("development".to_string()),
            command: "{{VENV_PATH}}/bin/python".to_string(),
            args: vec!["{{SCRIPT_PATH}}".to_string()],
            env: HashMap::from([
                ("PYTHONPATH".to_string(), "{{PROJECT_PATH}}".to_string()),
            ]),
            default_cwd: Some("{{PROJECT_PATH}}".to_string()),
            default_auto_start: false,
            variables: vec![
                TemplateVariable {
                    name: "VENV_PATH".to_string(),
                    description: Some("Path to Python virtual environment".to_string()),
                    var_type: VariableType::Path,
                    default_value: Some(".venv".to_string()),
                    required: true,
                    validation: None,
                },
                TemplateVariable {
                    name: "SCRIPT_PATH".to_string(),
                    description: Some("Path to Python script".to_string()),
                    var_type: VariableType::Path,
                    default_value: None,
                    required: true,
                    validation: None,
                },
                TemplateVariable {
                    name: "PROJECT_PATH".to_string(),
                    description: Some("Path to the project directory".to_string()),
                    var_type: VariableType::Path,
                    default_value: None,
                    required: true,
                    validation: None,
                },
            ],
            tags: vec!["python".to_string(), "development".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        
        // ログ監視
        ProcessTemplate {
            id: None,
            template_id: "tail-logs".to_string(),
            name: "Log File Monitor".to_string(),
            description: Some("Monitor log file changes in real-time".to_string()),
            category: Some("monitoring".to_string()),
            command: "tail".to_string(),
            args: vec!["-f".to_string(), "{{LOG_FILE}}".to_string()],
            env: HashMap::new(),
            default_cwd: None,
            default_auto_start: false,
            variables: vec![
                TemplateVariable {
                    name: "LOG_FILE".to_string(),
                    description: Some("Path to the log file".to_string()),
                    var_type: VariableType::Path,
                    default_value: None,
                    required: true,
                    validation: None,
                },
            ],
            tags: vec!["monitoring".to_string(), "logs".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ]
}