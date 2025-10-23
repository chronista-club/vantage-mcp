use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generate a unique ID for templates and clipboard items
pub fn generate_id() -> String {
    nanoid::nanoid!()
}

/// Process state in the lifecycle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    #[default]
    NotStarted,
    Running,
    Stopped,
    Failed,
}

/// Process status including state and additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub state: ProcessState,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        ProcessStatus {
            state: ProcessState::NotStarted,
            pid: None,
            exit_code: None,
            started_at: None,
            stopped_at: None,
            error: None,
        }
    }
}

/// Process information stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Optional unique record identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Process unique identifier
    pub process_id: String,

    /// Display name for the process
    pub name: String,

    /// Command to execute
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Working directory
    pub cwd: Option<String>,

    /// Process status
    pub status: ProcessStatus,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Whether to auto-start on restore
    pub auto_start_on_restore: bool,
}

/// プロセステンプレート - よく使うプロセス設定を保存して再利用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTemplate {
    /// Optional unique record identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

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

    /// 作成日時
    pub created_at: DateTime<Utc>,

    /// 更新日時
    pub updated_at: DateTime<Utc>,

    /// タグ（検索・フィルタリング用）
    pub tags: Vec<String>,
}

impl ProcessTemplate {
    /// テンプレートから新しいプロセス情報を生成
    pub fn instantiate(
        &self,
        process_id: String,
        values: HashMap<String, String>,
    ) -> Result<ProcessInfo, String> {
        // 変数を置換
        let mut command = self.command.clone();
        let mut args = self.args.clone();
        let mut env = self.env.clone();

        // 変数置換処理
        for (key, value) in &values {
            let placeholder = format!("{{{{{key}}}}}");

            // コマンドの置換
            command = command.replace(&placeholder, value);

            // 引数の置換
            args = args
                .iter()
                .map(|arg| arg.replace(&placeholder, value))
                .collect();

            // 環境変数の置換
            for env_value in env.values_mut() {
                *env_value = env_value.replace(&placeholder, value);
            }
        }

        // 必須変数のチェック
        for var in &self.variables {
            if var.required && !values.contains_key(&var.name) {
                return Err(format!("Required variable '{}' is missing", var.name));
            }
        }

        Ok(ProcessInfo {
            id: None,
            process_id,
            name: self.name.clone(),
            command,
            args,
            env,
            cwd: self.default_cwd.clone(),
            status: ProcessStatus {
                state: ProcessState::NotStarted,
                pid: None,
                exit_code: None,
                started_at: None,
                stopped_at: None,
                error: None,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: self.tags.clone(),
            auto_start_on_restore: self.default_auto_start,
        })
    }
}

impl ProcessTemplate {
    /// 新しいテンプレートを作成
    pub fn new(name: String, command: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            template_id: generate_id(),
            name,
            description: None,
            category: None,
            command,
            args: Vec::new(),
            env: HashMap::new(),
            default_cwd: None,
            default_auto_start: false,
            variables: Vec::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

/// テンプレート変数の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// 変数名（${VAR_NAME} の形式で使用）
    pub name: String,

    /// 変数の説明
    pub description: Option<String>,

    /// デフォルト値
    pub default_value: Option<String>,

    /// 必須かどうか
    pub required: bool,

    /// 変数の型ヒント
    pub var_type: Option<String>,

    /// 値の例
    pub example: Option<String>,
}

/// クリップボードアイテム - ファイルやテキストの共有
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Optional unique record identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// 一意のID
    pub clipboard_id: String,

    /// 内容（テキストまたはBase64エンコードされたバイナリ）
    pub content: String,

    /// ファイル名（ファイルの場合）
    pub filename: Option<String>,

    /// 作成日時
    pub created_at: DateTime<Utc>,

    /// 更新日時
    pub updated_at: DateTime<Utc>,

    /// コンテンツタイプ（text, file, image など）
    pub content_type: Option<String>,

    /// タグ
    pub tags: Vec<String>,
}

impl ClipboardItem {
    /// 新しいクリップボードアイテムを作成
    pub fn new(content: String, filename: Option<String>, content_type: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            clipboard_id: generate_id(),
            content,
            filename,
            created_at: now,
            updated_at: now,
            content_type,
            tags: Vec::new(),
        }
    }
}

/// Settings stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub auto_save_interval: Option<u64>,
    pub max_log_lines: Option<usize>,
    pub enable_auto_restart: bool,
    pub default_shell: Option<String>,
    pub env_variables: HashMap<String, String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".to_string(),
            auto_save_interval: Some(300), // 5 minutes
            max_log_lines: Some(1000),
            enable_auto_restart: false,
            default_shell: None,
            env_variables: HashMap::new(),
            updated_at: Utc::now(),
        }
    }
}
