use std::io;
use thiserror::Error;

/// Ichimi Server の統一エラー型
#[derive(Debug, Error)]
pub enum IchimiError {
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
    #[cfg(feature = "web")]
    #[error("Web server error: {0}")]
    WebServer(String),

    #[cfg(feature = "web")]
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

/// Result型のエイリアス（IchimiResult として使用）
pub type IchimiResult<T> = std::result::Result<T, IchimiError>;

impl IchimiError {
    /// エラーをMCP用の文字列に変換
    pub fn to_mcp_error(&self) -> String {
        match self {
            IchimiError::ProcessNotFound(id) => format!("Process '{}' not found", id),
            IchimiError::ProcessAlreadyExists(id) => format!("Process '{}' already exists", id),
            IchimiError::SecurityValidation(msg) => format!("Security validation failed: {}", msg),
            IchimiError::CommandInjection(msg) => format!("Command injection detected: {}", msg),
            _ => self.to_string(),
        }
    }

    /// セキュリティエラーかどうかを判定
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            IchimiError::SecurityValidation(_)
                | IchimiError::CommandInjection(_)
                | IchimiError::InvalidPath(_)
                | IchimiError::PermissionDenied(_)
        )
    }

    /// リトライ可能なエラーかどうかを判定
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            IchimiError::Io(_) | IchimiError::DatabaseConnection(_) | IchimiError::Timeout(_)
        )
    }
}

// 文字列からのエラー変換（後方互換性のため）
impl From<String> for IchimiError {
    fn from(s: String) -> Self {
        IchimiError::Other(s)
    }
}

impl From<&str> for IchimiError {
    fn from(s: &str) -> Self {
        IchimiError::Other(s.to_string())
    }
}

// anyhow::Errorからの変換
impl From<anyhow::Error> for IchimiError {
    fn from(err: anyhow::Error) -> Self {
        IchimiError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IchimiError::ProcessNotFound("test-process".to_string());
        assert_eq!(err.to_string(), "Process not found: test-process");
    }

    #[test]
    fn test_security_error_detection() {
        let err = IchimiError::CommandInjection("rm -rf /".to_string());
        assert!(err.is_security_error());

        let err = IchimiError::ProcessNotFound("test".to_string());
        assert!(!err.is_security_error());
    }

    #[test]
    fn test_retryable_error() {
        let err = IchimiError::Timeout("operation".to_string());
        assert!(err.is_retryable());

        let err = IchimiError::ProcessAlreadyExists("test".to_string());
        assert!(!err.is_retryable());
    }
}
