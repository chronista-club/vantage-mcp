pub mod kdl_serde;
pub mod persistence;
pub mod snapshot;
pub mod types;

// Re-export main types
pub use persistence::manager::PersistenceManager;

// Re-export types for convenience
pub use types::{
    ClipboardItem, ProcessInfo, ProcessState, ProcessStatus, ProcessTemplate, Settings,
    TemplateVariable, generate_id,
};
