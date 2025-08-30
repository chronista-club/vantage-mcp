//! GitHub Actions CI監視機能
//!
//! gh CLIを使用してCI/CDパイプラインの状態を監視し、
//! 実行結果を追跡する機能を提供します。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, warn};

/// CI実行の状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CiRunStatus {
    Queued,
    InProgress,
    Completed,
}

/// CI実行の結論
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CiRunConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    Neutral,
    Unknown,
}

/// CI実行情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiRun {
    pub id: u64,
    pub name: String,
    pub workflow_name: String,
    pub branch: String,
    pub event: String,
    pub status: CiRunStatus,
    pub conclusion: Option<CiRunConclusion>,
    pub created_at: String,
    pub updated_at: String,
    pub duration: Option<String>,
    pub url: String,
}

/// CI監視マネージャー
#[derive(Clone)]
pub struct CiMonitor {
    /// 監視中のCI実行
    runs: Arc<RwLock<HashMap<u64, CiRun>>>,
    /// リポジトリパス
    repo_path: Option<String>,
    /// ポーリング間隔（秒）
    poll_interval: u64,
}

impl CiMonitor {
    /// 新しいCI監視マネージャーを作成
    pub fn new(repo_path: Option<String>, poll_interval: Option<u64>) -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            repo_path,
            poll_interval: poll_interval.unwrap_or(30),
        }
    }

    /// 最新のCI実行を取得
    pub async fn get_latest_runs(&self, limit: usize) -> Result<Vec<CiRun>, String> {
        let mut cmd = Command::new("gh");
        cmd.arg("run")
            .arg("list")
            .arg("--limit")
            .arg(limit.to_string())
            .arg("--json")
            .arg("databaseId,name,workflowName,headBranch,event,status,conclusion,createdAt,updatedAt,displayTitle,url")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(repo) = &self.repo_path {
            cmd.arg("--repo").arg(repo);
        }

        let output = cmd.output().await.map_err(|e| {
            error!("Failed to execute gh command: {}", e);
            format!("Failed to execute gh command: {}", e)
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("gh command failed: {}", stderr);
            return Err(format!("gh command failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let raw_runs: Vec<serde_json::Value> = serde_json::from_str(&stdout).map_err(|e| {
            error!("Failed to parse gh output: {}", e);
            format!("Failed to parse gh output: {}", e)
        })?;

        let runs: Vec<CiRun> = raw_runs
            .into_iter()
            .map(|run| {
                let status = match run["status"].as_str() {
                    Some("queued") => CiRunStatus::Queued,
                    Some("in_progress") => CiRunStatus::InProgress,
                    Some("completed") => CiRunStatus::Completed,
                    _ => CiRunStatus::Queued,
                };

                let conclusion = run["conclusion"].as_str().map(|c| match c {
                    "success" => CiRunConclusion::Success,
                    "failure" => CiRunConclusion::Failure,
                    "cancelled" => CiRunConclusion::Cancelled,
                    "skipped" => CiRunConclusion::Skipped,
                    "timed_out" => CiRunConclusion::TimedOut,
                    "action_required" => CiRunConclusion::ActionRequired,
                    "neutral" => CiRunConclusion::Neutral,
                    _ => CiRunConclusion::Unknown,
                });

                CiRun {
                    id: run["databaseId"].as_u64().unwrap_or(0),
                    name: run["displayTitle"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string(),
                    workflow_name: run["workflowName"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string(),
                    branch: run["headBranch"].as_str().unwrap_or("Unknown").to_string(),
                    event: run["event"].as_str().unwrap_or("Unknown").to_string(),
                    status,
                    conclusion,
                    created_at: run["createdAt"].as_str().unwrap_or("").to_string(),
                    updated_at: run["updatedAt"].as_str().unwrap_or("").to_string(),
                    duration: None, // TODO: Calculate from timestamps
                    url: run["url"].as_str().unwrap_or("").to_string(),
                }
            })
            .collect();

        // キャッシュを更新
        let mut cache = self.runs.write().await;
        for run in &runs {
            cache.insert(run.id, run.clone());
        }

        Ok(runs)
    }

    /// 特定のCI実行の詳細を取得
    pub async fn get_run_details(&self, run_id: u64) -> Result<String, String> {
        let mut cmd = Command::new("gh");
        cmd.arg("run")
            .arg("view")
            .arg(run_id.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(repo) = &self.repo_path {
            cmd.arg("--repo").arg(repo);
        }

        let output = cmd.output().await.map_err(|e| {
            error!("Failed to execute gh command: {}", e);
            format!("Failed to execute gh command: {}", e)
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("gh command failed: {}", stderr);
            return Err(format!("gh command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 失敗したCI実行のログを取得
    pub async fn get_failed_logs(&self, run_id: u64) -> Result<String, String> {
        let mut cmd = Command::new("gh");
        cmd.arg("run")
            .arg("view")
            .arg(run_id.to_string())
            .arg("--log-failed")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(repo) = &self.repo_path {
            cmd.arg("--repo").arg(repo);
        }

        let output = cmd.output().await.map_err(|e| {
            error!("Failed to execute gh command: {}", e);
            format!("Failed to execute gh command: {}", e)
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // 実行中の場合はエラーではなく情報として扱う
            if stderr.contains("still in progress") {
                info!("Run {} is still in progress", run_id);
                return Ok(
                    "CI run is still in progress. Logs will be available when it completes."
                        .to_string(),
                );
            }
            error!("gh command failed: {}", stderr);
            return Err(format!("gh command failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// CI実行を監視し、完了を待つ
    pub async fn wait_for_completion(
        &self,
        run_id: u64,
        timeout_secs: Option<u64>,
    ) -> Result<CiRun, String> {
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(600)); // デフォルト10分
        let mut interval_timer = interval(Duration::from_secs(self.poll_interval));
        let start = tokio::time::Instant::now();

        info!("Waiting for CI run {} to complete", run_id);

        loop {
            interval_timer.tick().await;

            if start.elapsed() > timeout {
                warn!("Timeout waiting for CI run {} to complete", run_id);
                return Err(format!("Timeout waiting for CI run {} to complete", run_id));
            }

            // 最新の状態を取得
            let runs = self.get_latest_runs(50).await?;

            if let Some(run) = runs.iter().find(|r| r.id == run_id) {
                debug!("CI run {} status: {:?}", run_id, run.status);

                if run.status == CiRunStatus::Completed {
                    info!(
                        "CI run {} completed with conclusion: {:?}",
                        run_id, run.conclusion
                    );
                    return Ok(run.clone());
                }
            } else {
                warn!("CI run {} not found in recent runs", run_id);
            }
        }
    }

    /// バックグラウンドでCI実行を監視
    pub async fn start_monitoring(&self) {
        let runs = self.runs.clone();
        let poll_interval = self.poll_interval;
        let repo_path = self.repo_path.clone();

        tokio::spawn(async move {
            let monitor = CiMonitor::new(repo_path, Some(poll_interval));
            let mut interval_timer = interval(Duration::from_secs(poll_interval));

            loop {
                interval_timer.tick().await;

                match monitor.get_latest_runs(10).await {
                    Ok(latest_runs) => {
                        let mut cache = runs.write().await;
                        for run in latest_runs {
                            let existing = cache.get(&run.id);

                            // 状態が変わった場合にログ出力
                            if let Some(existing_run) = existing {
                                if existing_run.status != run.status {
                                    info!(
                                        "CI run {} status changed: {:?} -> {:?}",
                                        run.id, existing_run.status, run.status
                                    );

                                    if run.status == CiRunStatus::Completed {
                                        info!(
                                            "CI run {} completed with conclusion: {:?}",
                                            run.id, run.conclusion
                                        );
                                    }
                                }
                            }

                            cache.insert(run.id, run);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get latest CI runs: {}", e);
                    }
                }
            }
        });

        info!("Started CI monitoring with {}s interval", poll_interval);
    }

    /// キャッシュされたCI実行を取得
    pub async fn get_cached_runs(&self) -> Vec<CiRun> {
        let runs = self.runs.read().await;
        runs.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ci_monitor_creation() {
        let monitor = CiMonitor::new(None, Some(60));
        assert_eq!(monitor.poll_interval, 60);
        assert!(monitor.repo_path.is_none());
    }

    #[tokio::test]
    async fn test_ci_status_parsing() {
        // CiRunStatus のパースをテスト
        assert_eq!(
            serde_json::from_str::<CiRunStatus>("\"Queued\"").unwrap(),
            CiRunStatus::Queued
        );
        assert_eq!(
            serde_json::from_str::<CiRunStatus>("\"InProgress\"").unwrap(),
            CiRunStatus::InProgress
        );
        assert_eq!(
            serde_json::from_str::<CiRunStatus>("\"Completed\"").unwrap(),
            CiRunStatus::Completed
        );
    }

    #[tokio::test]
    async fn test_ci_conclusion_parsing() {
        // CiRunConclusion のパースをテスト
        assert_eq!(
            serde_json::from_str::<CiRunConclusion>("\"Success\"").unwrap(),
            CiRunConclusion::Success
        );
        assert_eq!(
            serde_json::from_str::<CiRunConclusion>("\"Failure\"").unwrap(),
            CiRunConclusion::Failure
        );
    }
}
