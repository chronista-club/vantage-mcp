use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSuggestionsRequest {
    #[serde(default)]
    pub current_process: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApplySuggestionRequest {
    pub suggestion_index: usize,
}
