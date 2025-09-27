use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Set clipboard content (text)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetClipboardTextRequest {
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Set clipboard content (file)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetClipboardFileRequest {
    pub content: String,
    pub filename: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Get latest clipboard content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetClipboardRequest {
    // Empty for now - might add filtering options later
}

/// Get all clipboard history
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetClipboardHistoryRequest {
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Search clipboard items
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchClipboardRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Clear clipboard
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClearClipboardRequest {
    // Empty - clears all clipboard content
}

/// Response for clipboard operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClipboardResponse {
    pub id: String,
    pub content: String,
    pub filename: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub content_type: String,
    pub tags: Vec<String>,
}

/// Response for clipboard history
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClipboardHistoryResponse {
    pub total_count: usize,
    pub items: Vec<ClipboardResponse>,
}
