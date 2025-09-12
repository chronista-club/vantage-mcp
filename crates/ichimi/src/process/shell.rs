use chrono::Utc;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use super::buffer::CircularBuffer;
use super::protocol::Process;
use super::types::{OutputStream, ProcessInfo, ProcessState};

/// Shell process implementation
pub struct ShellProcess {
    pub info: ProcessInfo,
    pub stdout_buffer: CircularBuffer,
    pub stderr_buffer: CircularBuffer,
    pub child: Option<Child>,
    pub output_handles: Option<(JoinHandle<()>, JoinHandle<()>)>,
}

impl ShellProcess {
    /// Create a new shell process
    pub fn new(
        id: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<PathBuf>,
    ) -> Self {
        Self {
            info: ProcessInfo {
                id,
                command,
                args,
                env,
                cwd,
                state: ProcessState::NotStarted,
                auto_start_on_restore: false,
            },
            stdout_buffer: CircularBuffer::new(1000),
            stderr_buffer: CircularBuffer::new(1000),
            child: None,
            output_handles: None,
        }
    }

    /// Builder for creating a shell process
    pub fn builder() -> ShellProcessBuilder {
        ShellProcessBuilder::default()
    }

    async fn start_impl(&mut self) -> Result<(), String> {
        // Check if already running
        if self.is_running() {
            return Err("Process is already running".to_string());
        }

        info!("Starting process: {}", self.info.id);

        // Build command
        let mut cmd = Command::new(&self.info.command);
        cmd.args(&self.info.args);

        // Set environment variables
        for (key, value) in &self.info.env {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(cwd) = &self.info.cwd {
            cmd.current_dir(cwd);
        }

        // Configure I/O
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        // Get the PID
        let pid = child
            .id()
            .ok_or_else(|| "Failed to get process PID".to_string())?;

        // Set up output capture
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        // Create stdout capture task
        let stdout_buffer = self.stdout_buffer.clone();
        let stdout_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    stdout_buffer.push(trimmed.to_string()).await;
                }
                line.clear();
            }
        });

        // Create stderr capture task
        let stderr_buffer = self.stderr_buffer.clone();
        let stderr_handle = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();

            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if !trimmed.is_empty() {
                    stderr_buffer.push(trimmed.to_string()).await;
                }
                line.clear();
            }
        });

        // Update state
        self.info.state = ProcessState::Running {
            pid,
            started_at: Utc::now(),
        };
        self.child = Some(child);
        self.output_handles = Some((stdout_handle, stderr_handle));

        info!("Process {} started with PID {}", self.info.id, pid);
        Ok(())
    }

    async fn stop_impl(&mut self) -> Result<(), String> {
        if !self.is_running() {
            return Err("Process is not running".to_string());
        }

        info!("Stopping process: {}", self.info.id);

        if let Some(mut child) = self.child.take() {
            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                if let Some(pid) = child.id() {
                    let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }

            // Wait for process to exit or timeout
            let timeout = tokio::time::Duration::from_secs(5);
            match tokio::time::timeout(timeout, child.wait()).await {
                Ok(Ok(status)) => {
                    let exit_code = status.code();
                    self.info.state = ProcessState::Stopped {
                        exit_code,
                        stopped_at: Utc::now(),
                    };
                    info!(
                        "Process {} stopped with exit code: {:?}",
                        self.info.id, exit_code
                    );
                }
                _ => {
                    // Force kill if timeout or error
                    let _ = child.kill().await;
                    self.info.state = ProcessState::Stopped {
                        exit_code: None,
                        stopped_at: Utc::now(),
                    };
                    warn!("Process {} was forcefully killed", self.info.id);
                }
            }

            // Clean up output handles
            if let Some((stdout_handle, stderr_handle)) = self.output_handles.take() {
                stdout_handle.abort();
                stderr_handle.abort();
            }

            Ok(())
        } else {
            Err("No child process found".to_string())
        }
    }

    async fn kill_impl(&mut self) -> Result<(), String> {
        if !self.is_running() {
            return Err("Process is not running".to_string());
        }

        info!("Force killing process: {}", self.info.id);

        if let Some(mut child) = self.child.take() {
            child
                .kill()
                .await
                .map_err(|e| format!("Failed to kill process: {}", e))?;

            self.info.state = ProcessState::Stopped {
                exit_code: None,
                stopped_at: Utc::now(),
            };

            // Clean up output handles
            if let Some((stdout_handle, stderr_handle)) = self.output_handles.take() {
                stdout_handle.abort();
                stderr_handle.abort();
            }

            info!("Process {} forcefully killed", self.info.id);
            Ok(())
        } else {
            Err("No child process found".to_string())
        }
    }

    async fn get_output_impl(&self, stream: OutputStream, lines: Option<usize>) -> Vec<String> {
        match stream {
            OutputStream::Stdout => {
                if let Some(n) = lines {
                    self.stdout_buffer.get_last_n(n).await
                } else {
                    self.stdout_buffer.get_all().await
                }
            }
            OutputStream::Stderr => {
                if let Some(n) = lines {
                    self.stderr_buffer.get_last_n(n).await
                } else {
                    self.stderr_buffer.get_all().await
                }
            }
            OutputStream::Both => {
                let stdout = self.stdout_buffer.get_all().await;
                let stderr = self.stderr_buffer.get_all().await;
                let mut combined = Vec::new();
                combined.extend(stdout);
                combined.extend(stderr);
                
                if let Some(n) = lines {
                    combined.into_iter().rev().take(n).rev().collect()
                } else {
                    combined
                }
            }
        }
    }

    async fn clear_output_impl(&mut self, stream: OutputStream) {
        match stream {
            OutputStream::Stdout => {
                self.stdout_buffer.clear().await;
            }
            OutputStream::Stderr => {
                self.stderr_buffer.clear().await;
            }
            OutputStream::Both => {
                self.stdout_buffer.clear().await;
                self.stderr_buffer.clear().await;
            }
        }
    }
}

impl Process for ShellProcess {
    fn id(&self) -> &str {
        &self.info.id
    }

    fn info(&self) -> &ProcessInfo {
        &self.info
    }

    fn info_mut(&mut self) -> &mut ProcessInfo {
        &mut self.info
    }

    fn state(&self) -> &ProcessState {
        &self.info.state
    }

    fn start(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(self.start_impl())
    }

    fn stop(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(self.stop_impl())
    }

    fn kill(&mut self) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(self.kill_impl())
    }

    fn get_output(&self, stream: OutputStream, lines: Option<usize>) 
        -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(self.get_output_impl(stream, lines))
    }

    fn clear_output(&mut self, stream: OutputStream) 
        -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(self.clear_output_impl(stream))
    }
}

/// Builder for ShellProcess
#[derive(Default)]
pub struct ShellProcessBuilder {
    id: Option<String>,
    command: Option<String>,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<PathBuf>,
    auto_start_on_restore: bool,
}

impl ShellProcessBuilder {
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn command(mut self, command: String) -> Self {
        self.command = Some(command);
        self
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn cwd(mut self, cwd: Option<PathBuf>) -> Self {
        self.cwd = cwd;
        self
    }

    pub fn auto_start_on_restore(mut self, auto_start: bool) -> Self {
        self.auto_start_on_restore = auto_start;
        self
    }

    pub fn build(self) -> Result<ShellProcess, String> {
        let id = self.id.ok_or_else(|| "Process ID is required".to_string())?;
        let command = self
            .command
            .ok_or_else(|| "Command is required".to_string())?;

        let mut process = ShellProcess::new(id, command, self.args, self.env, self.cwd);
        process.info.auto_start_on_restore = self.auto_start_on_restore;
        Ok(process)
    }
}