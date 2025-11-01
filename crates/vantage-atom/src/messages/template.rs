use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// テンプレート作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
}

/// テンプレート更新リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTemplateRequest {
    pub id: String,
    pub name: Option<String>,
    pub command: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
}

/// テンプレート取得リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTemplateRequest {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// テンプレート削除リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteTemplateRequest {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// テンプレート一覧リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListTemplatesRequest {
    pub category: Option<String>,
    pub tag: Option<String>,
}

/// テンプレートからプロセス作成リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateProcessFromTemplateRequest {
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub process_id: String,
    pub override_args: Option<Vec<String>>,
    pub override_env: Option<HashMap<String, String>>,
    pub override_cwd: Option<String>,
    pub auto_start: Option<bool>,
}
