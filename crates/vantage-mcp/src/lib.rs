//! Vantage MCP (Model Context Protocol) クレート
//!
//! MCPプロトコル関連の型定義と実装を提供します。

pub mod error;

pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // 基本的なテスト
        assert_eq!(2 + 2, 4);
    }
}
