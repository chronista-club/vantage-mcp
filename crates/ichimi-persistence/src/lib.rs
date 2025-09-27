pub mod persistence;
pub mod types;
pub mod yaml;

// Re-export main types
pub use persistence::manager::PersistenceManager;

// Re-export types for convenience
pub use types::{
    ProcessTemplate, TemplateVariable, ClipboardItem,
    ProcessInfo, ProcessState, ProcessStatus,
    Settings, generate_id,
};