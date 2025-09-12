pub mod db;
pub mod persistence;
pub mod types;

// Re-export main types
pub use db::Database;
pub use persistence::manager::PersistenceManager;

// Re-export types for convenience
pub use types::{
    ProcessTemplate, TemplateVariable, ClipboardItem,
    ProcessInfo, ProcessState, ProcessStatus,
    Settings, generate_id,
};