use super::buffer::CircularBuffer;
use super::types::*;
use crate::persistence::PersistenceManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::info;

/// 管理されるプロセス
pub struct ManagedProcess {
    pub info: ProcessInfo,
    pub stdout_buffer: CircularBuffer,
    pub stderr_buffer: CircularBuffer,
    pub child: Option<Child>,
    pub output_handles: Option<(JoinHandle<()>, JoinHandle<()>)>,
}

impl ManagedProcess {
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
                auto_start: false,
            },
            stdout_buffer: CircularBuffer::new(1000),
            stderr_buffer: CircularBuffer::new(1000),
            child: None,
            output_handles: None,
        }
    }
}

/// プロセスマネージャー
#[derive(Clone)]
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<String, Arc<RwLock<ManagedProcess>>>>>,
    persistence: Arc<PersistenceManager>,
}

impl ProcessManager {
    pub async fn new() -> Self {
        let persistence = match PersistenceManager::new().await {
            Ok(pm) => Arc::new(pm),
            Err(e) => {
                tracing::error!("Failed to initialize persistence: {}", e);
                panic!("Cannot continue without persistence layer");
            }
        };

        let manager = Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            persistence: persistence.clone(),
        };

        // Load persisted processes on startup
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager_clone.load_persisted_processes().await {
                tracing::warn!("Failed to load persisted processes: {}", e);
            }
        });

        // Start auto-export if enabled
        if let Ok(interval_str) = std::env::var("ICHIMI_AUTO_EXPORT_INTERVAL") {
            if let Ok(interval_secs) = interval_str.parse::<u64>() {
                if interval_secs > 0 {
                    let manager_clone = manager.clone();
                    tokio::spawn(async move {
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
                        loop {
                            interval.tick().await;
                            if let Err(e) = manager_clone.export_processes(None).await {
                                tracing::warn!("Auto-export failed: {}", e);
                            } else {
                                tracing::debug!("Auto-export completed");
                            }
                        }
                    });
                    tracing::info!("Auto-export enabled with interval: {}s", interval_secs);
                }
            }
        }

        manager
    }

    async fn load_persisted_processes(&self) -> Result<(), String> {
        let loaded_processes = self.persistence.load_all_processes().await?;
        let mut processes = self.processes.write().await;

        for (id, info) in loaded_processes {
            let managed = ManagedProcess {
                info,
                stdout_buffer: CircularBuffer::new(1000),
                stderr_buffer: CircularBuffer::new(1000),
                child: None,
                output_handles: None,
            };
            processes.insert(id, Arc::new(RwLock::new(managed)));
        }

        tracing::info!("Loaded {} persisted processes", processes.len());
        Ok(())
    }

    /// プロセスを作成・登録
    pub async fn create_process(
        &self,
        id: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<PathBuf>,
    ) -> Result<(), String> {
        let mut processes = self.processes.write().await;

        if processes.contains_key(&id) {
            return Err(format!("Process with id '{}' already exists", id));
        }

        let process = ManagedProcess::new(id.clone(), command, args, env, cwd);
        let process_info = process.info.clone();
        processes.insert(id, Arc::new(RwLock::new(process)));

        // Persist the process
        if let Err(e) = self.persistence.save_process(&process_info).await {
            tracing::warn!("Failed to persist process: {}", e);
        }

        Ok(())
    }

    /// プロセスを起動
    pub async fn start_process(&self, id: String) -> Result<u32, String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?
            .clone();
        drop(processes);

        let mut process = process_arc.write().await;

        // すでに実行中の場合はエラー
        if matches!(process.info.state, ProcessState::Running { .. }) {
            return Err(format!("Process '{}' is already running", id));
        }

        // コマンドを構築
        let mut cmd = Command::new(&process.info.command);
        cmd.args(&process.info.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // 環境変数を設定
        for (key, value) in &process.info.env {
            cmd.env(key, value);
        }

        // 作業ディレクトリを設定
        if let Some(cwd) = &process.info.cwd {
            cmd.current_dir(cwd);
        }

        // プロセスを起動
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start process: {}", e))?;

        let pid = child
            .id()
            .ok_or_else(|| "Failed to get process ID".to_string())?;

        // 標準出力と標準エラー出力を処理
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        let stdout_buffer = process.stdout_buffer.clone();
        let stderr_buffer = process.stderr_buffer.clone();

        // 出力を非同期で読み取る
        let stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                stdout_buffer.push(line).await;
            }
        });

        let stderr_handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                stderr_buffer.push(line).await;
            }
        });

        // プロセス情報を更新
        process.info.state = ProcessState::Running {
            pid,
            started_at: chrono::Utc::now(),
        };
        process.child = Some(child);
        process.output_handles = Some((stdout_handle, stderr_handle));

        // Persist the updated state
        if let Err(e) = self.persistence.update_process(&process.info).await {
            tracing::warn!("Failed to persist process state: {}", e);
        }

        info!("Started process '{}' with PID {}", id, pid);
        Ok(pid)
    }

    /// プロセスを停止
    pub async fn stop_process(
        &self,
        id: String,
        grace_period_ms: Option<u64>,
    ) -> Result<(), String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?
            .clone();
        drop(processes);

        let mut process = process_arc.write().await;

        // 実行中でない場合はエラー
        if !matches!(process.info.state, ProcessState::Running { .. }) {
            return Err(format!("Process '{}' is not running", id));
        }

        if let Some(mut child) = process.child.take() {
            // グレースフルシャットダウンを試みる
            if let Some(grace_ms) = grace_period_ms {
                child
                    .kill()
                    .await
                    .map_err(|e| format!("Failed to kill process: {}", e))?;

                // 指定時間待機
                let timeout = tokio::time::Duration::from_millis(grace_ms);
                let _ = tokio::time::timeout(timeout, child.wait()).await;
            } else {
                // 即座に終了
                child
                    .kill()
                    .await
                    .map_err(|e| format!("Failed to kill process: {}", e))?;
            }

            // 出力ハンドルをクリーンアップ
            if let Some((stdout_handle, stderr_handle)) = process.output_handles.take() {
                stdout_handle.abort();
                stderr_handle.abort();
            }

            // 状態を更新
            process.info.state = ProcessState::Stopped {
                exit_code: None,
                stopped_at: chrono::Utc::now(),
            };

            // Persist the updated state
            if let Err(e) = self.persistence.update_process(&process.info).await {
                tracing::warn!("Failed to persist process state: {}", e);
            }

            info!("Stopped process '{}'", id);
        }

        Ok(())
    }

    /// プロセスのステータスを取得
    pub async fn get_process_status(&self, id: String) -> Result<ProcessStatus, String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?;

        let process = process_arc.read().await;

        let uptime_seconds = match &process.info.state {
            ProcessState::Running { started_at, .. } => {
                Some((chrono::Utc::now() - *started_at).num_seconds() as u64)
            }
            _ => None,
        };

        Ok(ProcessStatus {
            info: process.info.clone(),
            cpu_usage: None,    // TODO: 実装
            memory_usage: None, // TODO: 実装
            uptime_seconds,
        })
    }

    /// プロセスの出力を取得
    pub async fn get_process_output(
        &self,
        id: String,
        stream: OutputStream,
        lines: Option<u32>,
    ) -> Result<Vec<String>, String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?;

        let process = process_arc.read().await;

        let n = lines.unwrap_or(100) as usize;

        let output = match stream {
            OutputStream::Stdout => process.stdout_buffer.get_last_n(n).await,
            OutputStream::Stderr => process.stderr_buffer.get_last_n(n).await,
            OutputStream::Both => {
                let mut combined = process.stdout_buffer.get_last_n(n / 2).await;
                combined.extend(process.stderr_buffer.get_last_n(n / 2).await);
                combined
            }
        };

        Ok(output)
    }

    /// すべてのプロセスをリスト
    pub async fn list_processes(&self, filter: Option<ProcessFilter>) -> Vec<ProcessInfo> {
        let processes = self.processes.read().await;
        let mut result = Vec::new();

        for process_arc in processes.values() {
            let process = process_arc.read().await;
            let info = &process.info;

            // フィルタリング
            if let Some(ref f) = filter {
                // 状態フィルタ
                if let Some(ref state_filter) = f.state {
                    let matches = match state_filter {
                        ProcessStateFilter::Running => {
                            matches!(info.state, ProcessState::Running { .. })
                        }
                        ProcessStateFilter::Stopped => {
                            matches!(info.state, ProcessState::Stopped { .. })
                        }
                        ProcessStateFilter::Failed => {
                            matches!(info.state, ProcessState::Failed { .. })
                        }
                        ProcessStateFilter::All => true,
                    };
                    if !matches {
                        continue;
                    }
                }

                // 名前パターンフィルタ
                if let Some(ref pattern) = f.name_pattern {
                    if !info.id.contains(pattern) && !info.command.contains(pattern) {
                        continue;
                    }
                }
            }

            result.push(info.clone());
        }

        result
    }

    /// プロセスを削除
    pub async fn remove_process(&self, id: String) -> Result<(), String> {
        // まず停止を試みる
        let _ = self.stop_process(id.clone(), Some(5000)).await;

        let mut processes = self.processes.write().await;
        processes
            .remove(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?;

        // Delete from persistence
        if let Err(e) = self.persistence.delete_process(&id).await {
            tracing::warn!("Failed to delete persisted process: {}", e);
        }

        Ok(())
    }

    /// Export processes to surql file
    pub async fn export_processes(&self, file_path: Option<String>) -> Result<String, String> {
        match file_path {
            Some(path) => {
                self.persistence.export_to_file(&path).await?;
                Ok(path)
            }
            None => self.persistence.export_default().await,
        }
    }

    /// Import processes from surql file
    pub async fn import_processes(&self, file_path: &str) -> Result<(), String> {
        // Import to database
        self.persistence.import_from_file(file_path).await?;

        // Reload processes into memory
        self.load_persisted_processes().await?;

        Ok(())
    }

    /// Update process configuration (e.g., auto_start flag)
    pub async fn update_process_config(
        &self,
        id: String,
        auto_start: Option<bool>,
    ) -> Result<(), String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{}' not found", id))?;

        let mut process = process_arc.write().await;
        
        // Update auto_start if provided
        if let Some(auto_start_value) = auto_start {
            process.info.auto_start = auto_start_value;
            info!("Updated process '{}' auto_start to {}", id, auto_start_value);
        }

        // Persist the updated configuration
        if let Err(e) = self.persistence.update_process(&process.info).await {
            return Err(format!("Failed to persist process config update: {}", e));
        }

        Ok(())
    }
}
