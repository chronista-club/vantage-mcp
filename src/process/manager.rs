use super::buffer::CircularBuffer;
use super::types::*;
use crate::db::Database;
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
                auto_start_on_create: false,
                auto_start_on_restore: false,
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
    database: Arc<Database>,
}

impl ProcessManager {
    pub async fn new() -> Self {
        let database = match Database::new().await {
            Ok(db) => Arc::new(db),
            Err(e) => {
                tracing::error!("Failed to initialize database: {}", e);
                panic!("Cannot continue without database");
            }
        };

        Self::with_database(database).await
    }

    /// Get the database instance
    pub fn database(&self) -> Arc<Database> {
        self.database.clone()
    }

    pub async fn with_database(database: Arc<Database>) -> Self {
        let persistence = Arc::new(PersistenceManager::with_database(database.clone()));

        let manager = Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            persistence: persistence.clone(),
            database,
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
        let mut auto_start_processes = Vec::new();

        for (id, info) in loaded_processes {
            // Check if this process should be auto-started on restore
            if info.auto_start_on_restore {
                auto_start_processes.push(id.clone());
            }

            let managed = ManagedProcess {
                info,
                stdout_buffer: CircularBuffer::new(1000),
                stderr_buffer: CircularBuffer::new(1000),
                child: None,
                output_handles: None,
            };
            processes.insert(id, Arc::new(RwLock::new(managed)));
        }

        let loaded_count = processes.len();
        tracing::info!("Loaded {} persisted processes", loaded_count);

        // Release the write lock before starting processes
        drop(processes);

        // Start auto-start processes
        if !auto_start_processes.is_empty() {
            tracing::info!(
                "Starting {} processes with auto_start_on_restore enabled",
                auto_start_processes.len()
            );
            for process_id in auto_start_processes {
                match self.start_process(process_id.clone()).await {
                    Ok(pid) => {
                        tracing::info!(
                            "Auto-started process '{}' with PID {} on restore",
                            process_id,
                            pid
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to auto-start process '{}' on restore: {}",
                            process_id,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// プロセスを作成・登録
    #[allow(clippy::too_many_arguments)]
    pub async fn create_process(
        &self,
        id: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<PathBuf>,
        auto_start_on_create: bool,
        auto_start_on_restore: bool,
    ) -> Result<(), String> {
        // セキュリティ検証
        crate::security::validate_process_inputs(&command, &args, &env, &cwd)?;

        info!(
            "Creating process '{}': {} {:?} (auto_start_on_create: {}, auto_start_on_restore: {})",
            id, command, args, auto_start_on_create, auto_start_on_restore
        );
        let mut processes = self.processes.write().await;

        if processes.contains_key(&id) {
            return Err(format!("Process with id '{id}' already exists"));
        }

        let mut process = ManagedProcess::new(id.clone(), command, args, env, cwd);
        process.info.auto_start_on_create = auto_start_on_create;
        process.info.auto_start_on_restore = auto_start_on_restore;

        let process_info = process.info.clone();
        let process_arc = Arc::new(RwLock::new(process));
        processes.insert(id.clone(), process_arc.clone());

        // Release the write lock before persistence and auto-start
        drop(processes);

        // Persist the process
        match self.persistence.save_process(&process_info).await {
            Ok(_) => tracing::debug!("Process {} persisted successfully", id),
            Err(e) => tracing::warn!("Failed to persist process {}: {}", id, e),
        }

        // Auto-start on create if configured
        if auto_start_on_create {
            info!("Auto-starting process '{}' on creation", id);
            if let Err(e) = self.start_process(id.clone()).await {
                tracing::warn!("Failed to auto-start process '{}': {}", id, e);
            }
        }

        Ok(())
    }

    /// プロセスを起動
    pub async fn start_process(&self, id: String) -> Result<u32, String> {
        info!("Starting process '{}'...", id);
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{id}' not found"))?
            .clone();
        drop(processes);

        let mut process = process_arc.write().await;

        // すでに実行中の場合はエラー
        if matches!(process.info.state, ProcessState::Running { .. }) {
            return Err(format!("Process '{id}' is already running"));
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
            .map_err(|e| format!("Failed to start process: {e}"))?;

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
        info!("Stopping process '{}'...", id);
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{id}' not found"))?
            .clone();
        drop(processes);

        let mut process = process_arc.write().await;

        // 実行中でない場合はエラー
        if !matches!(process.info.state, ProcessState::Running { .. }) {
            return Err(format!("Process '{id}' is not running"));
        }

        if let Some(mut child) = process.child.take() {
            // グレースフルシャットダウンを試みる
            if let Some(grace_ms) = grace_period_ms {
                child
                    .kill()
                    .await
                    .map_err(|e| format!("Failed to kill process: {e}"))?;

                // 指定時間待機
                let timeout = tokio::time::Duration::from_millis(grace_ms);
                let _ = tokio::time::timeout(timeout, child.wait()).await;
            } else {
                // 即座に終了
                child
                    .kill()
                    .await
                    .map_err(|e| format!("Failed to kill process: {e}"))?;
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
            .ok_or_else(|| format!("Process '{id}' not found"))?;

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
            .ok_or_else(|| format!("Process '{id}' not found"))?;

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
            .ok_or_else(|| format!("Process '{id}' not found"))?;

        // Delete from persistence
        if let Err(e) = self.persistence.delete_process(&id).await {
            tracing::warn!("Failed to delete persisted process: {}", e);
        }

        Ok(())
    }

    /// Export processes to surql file
    pub async fn export_processes(&self, file_path: Option<String>) -> Result<String, String> {
        let path = match file_path {
            Some(p) => std::path::PathBuf::from(p),
            None => Database::get_default_data_path(),
        };

        self.database
            .export_to_file(&path)
            .await
            .map_err(|e| format!("Failed to export processes: {e}"))?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Import processes from surql file
    pub async fn import_processes(&self, file_path: &str) -> Result<(), String> {
        let path = std::path::Path::new(file_path);

        // Import to database using Database's import method which handles .surql format
        self.database
            .import_from_file(path)
            .await
            .map_err(|e| format!("Failed to import processes: {e}"))?;

        // Reload processes into memory
        self.load_persisted_processes().await?;

        Ok(())
    }

    /// Update process configuration (auto_start flags)
    pub async fn update_process_config(
        &self,
        id: String,
        auto_start_on_create: Option<bool>,
        auto_start_on_restore: Option<bool>,
    ) -> Result<(), String> {
        let processes = self.processes.read().await;
        let process_arc = processes
            .get(&id)
            .ok_or_else(|| format!("Process '{id}' not found"))?;

        let mut process = process_arc.write().await;

        if let Some(value) = auto_start_on_create {
            process.info.auto_start_on_create = value;
            info!("Updated process '{}' auto_start_on_create to {}", id, value);
        }

        if let Some(value) = auto_start_on_restore {
            process.info.auto_start_on_restore = value;
            info!(
                "Updated process '{}' auto_start_on_restore to {}",
                id, value
            );
        }

        // Persist the updated configuration
        if let Err(e) = self.persistence.update_process(&process.info).await {
            return Err(format!("Failed to persist process config update: {e}"));
        }

        Ok(())
    }
}
