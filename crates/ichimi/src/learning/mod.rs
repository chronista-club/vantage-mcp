use anyhow::Result;
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

// Database removed - SurrealDB dependency eliminated
use crate::events::{EventSystem, EventType, ProcessEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPattern {
    pub process_id: String,
    pub next_processes: Vec<String>,
    pub confidence: f64,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePattern {
    pub hour_range: (u32, u32),
    pub day_of_week: Vec<u32>,
    pub processes: Vec<String>,
    pub frequency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub message: String,
    pub confidence: f64,
    pub action: SuggestedAction,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedAction {
    StartProcess { process_id: String },
    StopProcess { process_id: String },
    RestartProcess { process_id: String },
    CreateProcess { command: String, args: Vec<String> },
}

#[derive(Clone)]
pub struct LearningEngine {
    event_system: Arc<EventSystem>,
    patterns: Arc<RwLock<HashMap<String, ProcessPattern>>>,
}

impl LearningEngine {
    pub fn new(event_system: Arc<EventSystem>) -> Self {
        Self {
            event_system,
            patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_learning(&self) -> Result<()> {
        info!("Starting learning engine");

        // イベントシステムからのイベントを監視
        let mut receiver = self.event_system.subscribe();
        let learning_self = self.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let Err(e) = learning_self.process_event(event).await {
                    error!("Failed to process event: {}", e);
                }
            }
        });

        info!("Learning engine started");
        Ok(())
    }

    // live_query_loop removed - Database dependency eliminated

    async fn update_patterns(
        patterns: Arc<RwLock<HashMap<String, ProcessPattern>>>,
        process_id: &str,
        event_type: &str,
    ) {
        let mut patterns = patterns.write().await;

        // 既存のパターンを取得または新規作成
        let pattern = patterns
            .entry(process_id.to_string())
            .or_insert_with(|| ProcessPattern {
                process_id: process_id.to_string(),
                next_processes: Vec::new(),
                confidence: 0.5,
                context: HashMap::new(),
            });

        // イベントタイプに応じて信頼度を調整
        match event_type {
            "start" => {
                pattern.confidence = (pattern.confidence * 0.9 + 1.0 * 0.1).min(1.0);
            }
            "stop" => {
                pattern.confidence = (pattern.confidence * 0.95).max(0.1);
            }
            "error" => {
                pattern.confidence = (pattern.confidence * 0.8).max(0.1);
            }
            "recover" => {
                pattern.confidence = (pattern.confidence * 0.9 + 0.8 * 0.1).min(1.0);
            }
            _ => {}
        }

        debug!(
            "Updated pattern for {}: confidence = {}",
            process_id, pattern.confidence
        );
    }

    async fn process_event(&self, event: ProcessEvent) -> Result<()> {
        debug!("Processing event: {:?}", event);

        // イベントタイプに応じた学習
        match event.event_type {
            EventType::ProcessStarted => {
                self.learn_process_start(&event.process_id).await?;
            }
            EventType::ProcessStopped => {
                self.learn_process_stop(&event.process_id).await?;
            }
            EventType::ProcessError => {
                self.learn_process_error(&event.process_id, event.context)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn learn_process_start(&self, process_id: &str) -> Result<()> {
        // プロセス開始パターンを記録（メモリ内で管理）
        let mut patterns = self.patterns.write().await;

        let pattern = patterns
            .entry(process_id.to_string())
            .or_insert_with(|| ProcessPattern {
                process_id: process_id.to_string(),
                next_processes: Vec::new(),
                confidence: 0.5,
                context: HashMap::new(),
            });

        pattern.confidence = (pattern.confidence + 0.05).min(1.0);
        Ok(())
    }

    async fn learn_process_stop(&self, process_id: &str) -> Result<()> {
        // プロセス停止パターンを記録（メモリ内で管理）
        let mut patterns = self.patterns.write().await;

        if let Some(pattern) = patterns.get_mut(process_id) {
            pattern.confidence = (pattern.confidence - 0.02).max(0.0);
        }

        Ok(())
    }

    async fn learn_process_error(
        &self,
        process_id: &str,
        context: Option<serde_json::Value>,
    ) -> Result<()> {
        // エラーパターンを記録（メモリ内で管理）
        let mut patterns = self.patterns.write().await;

        if let Some(pattern) = patterns.get_mut(process_id) {
            pattern.confidence = (pattern.confidence - 0.1).max(0.0);
            if let Some(ctx) = context {
                pattern.context.insert("last_error".to_string(), ctx);
            }
        }

        Ok(())
    }

    pub async fn get_suggestions(&self, current_process: Option<&str>) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();

        // 現在のプロセスに基づいて次のプロセスを提案（メモリ内パターンから）
        if let Some(process_id) = current_process {
            let patterns = self.patterns.read().await;

            if let Some(pattern) = patterns.get(process_id) {
                if pattern.confidence > 0.6 {
                    for next_process in &pattern.next_processes {
                        suggestions.push(Suggestion {
                            message: format!(
                                "「{process_id}」が起動しました。通常は「{next_process}」も必要です。"
                            ),
                            confidence: pattern.confidence,
                            action: SuggestedAction::StartProcess {
                                process_id: next_process.clone(),
                            },
                            reason: format!(
                                "過去のパターンから学習（信頼度: {:.0}%）",
                                pattern.confidence * 100.0
                            ),
                        });
                    }
                }
            }
        }

        // 時間帯に基づく提案（簡易実装）
        // TODO: 実際の時間パターン学習を実装

        Ok(suggestions)
    }
}

// Clone is now derived automatically with #[derive(Clone)]
