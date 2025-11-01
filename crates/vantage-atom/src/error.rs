use std::io;
use thiserror::Error;

/// Vantage MCP の統一エラー型
#[derive(Debug, Error)]
pub enum VantageError {
    // プロセス管理エラー
    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Process already exists: {0}")]
    ProcessAlreadyExists(String),

    #[error("Process already running: {0}")]
    ProcessAlreadyRunning(String),

    #[error("Process not running: {0}")]
    ProcessNotRunning(String),

    #[error("Failed to start process: {0}")]
    ProcessStartFailed(String),

    #[error("Failed to stop process: {0}")]
    ProcessStopFailed(String),

    // セキュリティエラー
    #[error("Security validation failed: {0}")]
    SecurityValidation(String),

    #[error("Command injection detected: {0}")]
    CommandInjection(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    // データベースエラー
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    #[error("Data serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // I/Oエラー
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    // Webサーバーエラー
    #[error("Web server error: {0}")]
    WebServer(String),

    #[error("Port already in use: {0}")]
    PortInUse(u16),

    // 一般的なエラー
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("{0}")]
    Other(String),
}

/// Result型のエイリアス（VantageResult として使用）
pub type VantageResult<T> = std::result::Result<T, VantageError>;

impl VantageError {
    /// エラーをMCP用の文字列に変換
    pub fn to_mcp_error(&self) -> String {
        match self {
            VantageError::ProcessNotFound(id) => format!("Process '{id}' not found"),
            VantageError::ProcessAlreadyExists(id) => format!("Process '{id}' already exists"),
            VantageError::SecurityValidation(msg) => format!("Security validation failed: {msg}"),
            VantageError::CommandInjection(msg) => format!("Command injection detected: {msg}"),
            _ => self.to_string(),
        }
    }

    /// セキュリティエラーかどうかを判定
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            VantageError::SecurityValidation(_)
                | VantageError::CommandInjection(_)
                | VantageError::InvalidPath(_)
                | VantageError::PermissionDenied(_)
        )
    }

    /// リトライ可能なエラーかどうかを判定
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            VantageError::Io(_) | VantageError::DatabaseConnection(_) | VantageError::Timeout(_)
        )
    }
}

// 文字列からのエラー変換（後方互換性のため）
impl From<String> for VantageError {
    fn from(s: String) -> Self {
        VantageError::Other(s)
    }
}

impl From<&str> for VantageError {
    fn from(s: &str) -> Self {
        VantageError::Other(s.to_string())
    }
}

// anyhow::Errorからの変換
impl From<anyhow::Error> for VantageError {
    fn from(err: anyhow::Error) -> Self {
        VantageError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VantageError::ProcessNotFound("test-process".to_string());
        assert_eq!(err.to_string(), "Process not found: test-process");
    }

    #[test]
    fn test_security_error_detection() {
        let err = VantageError::CommandInjection("rm -rf /".to_string());
        assert!(err.is_security_error());

        let err = VantageError::ProcessNotFound("test".to_string());
        assert!(!err.is_security_error());
    }

    #[test]
    fn test_retryable_error() {
        let err = VantageError::Timeout("operation".to_string());
        assert!(err.is_retryable());

        let err = VantageError::ProcessAlreadyExists("test".to_string());
        assert!(!err.is_retryable());
    }
}
