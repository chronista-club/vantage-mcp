use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ProcessStarted,
    ProcessStopped,
    ProcessError,
    ProcessRecovered,
    ProcessCreated,
    ProcessRemoved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_type: EventType,
    pub process_id: String,
    pub timestamp: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

impl ProcessEvent {
    pub fn new(
        event_type: EventType,
        process_id: String,
        context: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            event_type,
            process_id,
            timestamp: Utc::now(),
            context,
            metadata,
        }
    }
}

#[derive(Clone)]
pub struct EventSystem {
    db: Arc<Database>,
    sender: broadcast::Sender<ProcessEvent>,
}

impl EventSystem {
    pub fn new(db: Arc<Database>) -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { db, sender }
    }

    pub async fn emit(&self, event: ProcessEvent) -> Result<()> {
        debug!("Emitting event: {:?}", event.event_type);

        // データベースに記録
        let event_type_str = match &event.event_type {
            EventType::ProcessStarted => "start",
            EventType::ProcessStopped => "stop",
            EventType::ProcessError => "error",
            EventType::ProcessRecovered => "recover",
            EventType::ProcessCreated => "create",
            EventType::ProcessRemoved => "remove",
        };

        self.db
            .record_event(
                event_type_str,
                &event.process_id,
                event.context.clone(),
                event.metadata.clone(),
            )
            .await?;

        // ブロードキャスト（リスナーがいなくてもエラーにしない）
        let _ = self.sender.send(event);

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ProcessEvent> {
        self.sender.subscribe()
    }

    pub async fn emit_process_started(&self, process_id: String, pid: Option<u32>) -> Result<()> {
        let mut context = serde_json::Map::new();
        if let Some(pid) = pid {
            context.insert("pid".to_string(), serde_json::Value::Number(pid.into()));
        }

        self.emit(ProcessEvent::new(
            EventType::ProcessStarted,
            process_id,
            Some(serde_json::Value::Object(context)),
            None,
        ))
        .await
    }

    pub async fn emit_process_stopped(
        &self,
        process_id: String,
        exit_code: Option<i32>,
    ) -> Result<()> {
        let mut context = serde_json::Map::new();
        if let Some(code) = exit_code {
            context.insert(
                "exit_code".to_string(),
                serde_json::Value::Number(code.into()),
            );
        }

        self.emit(ProcessEvent::new(
            EventType::ProcessStopped,
            process_id,
            Some(serde_json::Value::Object(context)),
            None,
        ))
        .await
    }

    pub async fn emit_process_error(&self, process_id: String, error: String) -> Result<()> {
        let mut context = serde_json::Map::new();
        context.insert("error".to_string(), serde_json::Value::String(error));

        self.emit(ProcessEvent::new(
            EventType::ProcessError,
            process_id,
            Some(serde_json::Value::Object(context)),
            None,
        ))
        .await
    }
}
