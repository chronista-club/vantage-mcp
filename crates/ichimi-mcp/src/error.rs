//! エラー型定義

use thiserror::Error;

/// Ichimi MCPのエラー型
#[derive(Error, Debug)]
pub enum Error {
    /// 一般的なエラー
    #[error("一般エラー: {0}")]
    General(String),

    /// シリアライゼーションエラー
    #[error("シリアライゼーションエラー: {0}")]
    Serialization(#[from] serde_json::Error),

    /// その他のエラー
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result型のエイリアス
pub type Result<T> = std::result::Result<T, Error>;
