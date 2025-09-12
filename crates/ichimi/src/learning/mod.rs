use anyhow::{Context, Result};
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use ichimi_persistence::Database;
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

pub struct LearningEngine {
    db: Arc<Database>,
    event_system: Arc<EventSystem>,
    patterns: Arc<RwLock<HashMap<String, ProcessPattern>>>,
}

impl LearningEngine {
    pub fn new(db: Arc<Database>, event_system: Arc<EventSystem>) -> Self {
        Self {
            db,
            event_system,
            patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_learning(&self) -> Result<()> {
        info!("Starting learning engine with LIVE QUERY");

        // LIVE QUERYを開始
        let db = self.db.clone();
        let patterns = self.patterns.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::live_query_loop(db, patterns).await {
                error!("LIVE QUERY loop failed: {}", e);
            }
        });

        // イベントシステムからのイベントも監視
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

    async fn live_query_loop(
        db: Arc<Database>,
        patterns: Arc<RwLock<HashMap<String, ProcessPattern>>>,
    ) -> Result<()> {
        let client = db.client().await;

        // LIVE QUERYを設定
        let mut response = client
            .query("LIVE SELECT * FROM process_event")
            .await
            .context("Failed to setup LIVE QUERY")?;

        // ストリームから結果を取得
        while let Ok(result) = response.take::<Vec<serde_json::Value>>(0usize) {
            debug!("Received LIVE QUERY event: {:?}", result);

            // パターンを更新
            for item in result {
                if let Some(event_obj) = item.as_object() {
                    if let (Some(process_id), Some(event_type)) =
                        (event_obj.get("process_id"), event_obj.get("type"))
                    {
                        let process_id = process_id.as_str().unwrap_or_default();
                        let event_type = event_type.as_str().unwrap_or_default();

                        // パターンを学習
                        Self::update_patterns(patterns.clone(), process_id, event_type).await;
                    }
                }
            }
        }

        Ok(())
    }

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
        // プロセス開始パターンを記録
        let client = self.db.client().await;

        let query = r#"
            UPDATE process_pattern:$id SET
                confidence = math::min(confidence + 0.05, 1.0)
            WHERE process_id = $process_id
        "#;

        client
            .query(query)
            .bind(("id", format!("pattern_{process_id}")))
            .bind(("process_id", process_id.to_string()))
            .await?;

        Ok(())
    }

    async fn learn_process_stop(&self, process_id: &str) -> Result<()> {
        // プロセス停止パターンを記録
        let client = self.db.client().await;

        let query = r#"
            UPDATE process_pattern:$id SET
                confidence = math::max(confidence - 0.02, 0.0)
            WHERE process_id = $process_id
        "#;

        client
            .query(query)
            .bind(("id", format!("pattern_{process_id}")))
            .bind(("process_id", process_id.to_string()))
            .await?;

        Ok(())
    }

    async fn learn_process_error(
        &self,
        process_id: &str,
        context: Option<serde_json::Value>,
    ) -> Result<()> {
        // エラーパターンを記録
        let client = self.db.client().await;

        let query = r#"
            UPDATE process_pattern:$id SET
                confidence = math::max(confidence - 0.1, 0.0),
                context.last_error = $error_context
            WHERE process_id = $process_id
        "#;

        client
            .query(query)
            .bind(("id", format!("pattern_{process_id}")))
            .bind(("process_id", process_id.to_string()))
            .bind(("error_context", context))
            .await?;

        Ok(())
    }

    pub async fn get_suggestions(&self, current_process: Option<&str>) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();
        let client = self.db.client().await;

        // 現在のプロセスに基づいて次のプロセスを提案
        if let Some(process_id) = current_process {
            let query = r#"
                SELECT *,
                    ->depends_on->process AS dependencies
                FROM process:$process_id
                WHERE confidence > 0.6
                ORDER BY confidence DESC
                LIMIT 3
            "#;

            let mut response = client
                .query(query)
                .bind(("process_id", process_id.to_string()))
                .await?;

            if let Ok(patterns) = response.take::<Vec<ProcessPattern>>(0) {
                for pattern in patterns {
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

        // 時間帯に基づく提案
        let current_hour = chrono::Local::now().hour();
        let current_day = chrono::Local::now().weekday().num_days_from_monday();

        let query = r#"
            SELECT * FROM time_pattern
            WHERE $hour >= hour_range[0] AND $hour <= hour_range[1]
                AND $day IN day_of_week
            ORDER BY frequency DESC
            LIMIT 3
        "#;

        let mut response = client
            .query(query)
            .bind(("hour", current_hour))
            .bind(("day", current_day))
            .await?;

        if let Ok(time_patterns) = response.take::<Vec<TimePattern>>(0) {
            for pattern in time_patterns {
                for process in &pattern.processes {
                    suggestions.push(Suggestion {
                        message: format!("この時間帯は通常「{process}」を起動しています。"),
                        confidence: 0.7,
                        action: SuggestedAction::StartProcess {
                            process_id: process.clone(),
                        },
                        reason: format!("時間帯パターン（頻度: {}回）", pattern.frequency),
                    });
                }
            }
        }

        Ok(suggestions)
    }
}

impl Clone for LearningEngine {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            event_system: self.event_system.clone(),
            patterns: self.patterns.clone(),
        }
    }
}
