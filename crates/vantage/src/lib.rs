//! Vantage - A powerful process management server for Claude Code via MCP
//!
//! This crate provides a high-level interface to the Vantage system.
//! The core functionality is provided by the `atom` module (vantage-atom crate).

// Re-export vantage-atom as atom module
pub use vantage_atom as atom;

// Re-export commonly used types at the root level for convenience
pub use atom::VantageServer;

// Optionally re-export other frequently used types
pub mod process {
    pub use crate::atom::process::*;
}

pub mod web {
    pub use crate::atom::web::*;
}

// Re-export error types
pub use atom::VantageError;
pub type Result<T> = std::result::Result<T, VantageError>;