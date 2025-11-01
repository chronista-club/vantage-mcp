//! CI監視関連のメッセージ型定義

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// CI実行リスト取得リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListCiRunsRequest {
    /// 取得する実行数の上限
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// リポジトリパス（省略時は現在のリポジトリ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

fn default_limit() -> usize {
    10
}

/// CI実行詳細取得リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetCiRunDetailsRequest {
    /// 実行ID
    pub run_id: u64,

    /// リポジトリパス（省略時は現在のリポジトリ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// 失敗ログ取得リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetCiFailedLogsRequest {
    /// 実行ID
    pub run_id: u64,

    /// リポジトリパス（省略時は現在のリポジトリ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

/// CI実行完了待機リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaitForCiCompletionRequest {
    /// 実行ID
    pub run_id: u64,

    /// タイムアウト（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// リポジトリパス（省略時は現在のリポジトリ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

fn default_timeout() -> u64 {
    600 // 10分
}

/// CI監視開始リクエスト
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StartCiMonitoringRequest {
    /// ポーリング間隔（秒）
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,

    /// リポジトリパス（省略時は現在のリポジトリ）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
}

fn default_poll_interval() -> u64 {
    30
}

/// CI実行レスポンス
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CiRunResponse {
    pub id: u64,
    pub name: String,
    pub workflow_name: String,
    pub branch: String,
    pub event: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub duration: Option<String>,
    pub url: String,
}

/// CI実行リストレスポンス
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListCiRunsResponse {
    pub runs: Vec<CiRunResponse>,
    pub total_count: usize,
}
