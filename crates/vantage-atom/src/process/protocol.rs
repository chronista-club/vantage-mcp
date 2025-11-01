use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use super::types::{OutputStream, ProcessInfo, ProcessState};

/// Process trait that defines the interface for all process types
pub trait Process: Send + Sync {
    /// Get the process ID
    fn id(&self) -> &str;

    /// Get the process information
    fn info(&self) -> &ProcessInfo;

    /// Get mutable process information
    fn info_mut(&mut self) -> &mut ProcessInfo;

    /// Get the current state of the process
    fn state(&self) -> &ProcessState;

    /// Start the process
    fn start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// Stop the process
    fn stop(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// Force kill the process
    fn kill(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// Get process output
    fn get_output(
        &self,
        stream: OutputStream,
        lines: Option<usize>,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;

    /// Clear output buffers
    fn clear_output(
        &mut self,
        stream: OutputStream,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    /// Check if the process is running
    fn is_running(&self) -> bool {
        matches!(self.state(), ProcessState::Running { .. })
    }

    /// Check if the process has stopped
    fn is_stopped(&self) -> bool {
        matches!(self.state(), ProcessState::Stopped { .. })
    }

    /// Check if the process has failed
    fn is_failed(&self) -> bool {
        matches!(self.state(), ProcessState::Failed { .. })
    }

    /// Check if the process has not started
    fn is_not_started(&self) -> bool {
        matches!(self.state(), ProcessState::NotStarted)
    }
}

/// Builder trait for creating processes
pub trait ProcessBuilder {
    /// Set the process ID
    fn id(self, id: String) -> Self;

    /// Set the command
    fn command(self, command: String) -> Self;

    /// Set the arguments
    fn args(self, args: Vec<String>) -> Self;

    /// Set environment variables
    fn env(self, env: HashMap<String, String>) -> Self;

    /// Set the working directory
    fn cwd(self, cwd: Option<PathBuf>) -> Self;

    /// Set auto-start on restore
    fn auto_start_on_restore(self, auto_start: bool) -> Self;

    /// Build the process
    fn build(self) -> Result<Box<dyn Process>, String>;
}
