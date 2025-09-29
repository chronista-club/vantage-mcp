use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::debug;

use super::models::*;

/// Clipboard queries
pub struct ClipboardQueries;

impl ClipboardQueries {
    /// Store a clipboard entry
    pub async fn store(
        pool: &SqlitePool,
        key: &str,
        content: &str,
        content_type: &str,
        metadata: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<i64> {
        let now = Utc::now();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO clipboard (key, content, content_type, metadata, created_at, updated_at, expires_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(key) DO UPDATE SET
                content = excluded.content,
                content_type = excluded.content_type,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at,
                expires_at = excluded.expires_at
            RETURNING id
            "#
        )
        .bind(key)
        .bind(content)
        .bind(content_type)
        .bind(metadata)
        .bind(now)
        .bind(now)
        .bind(expires_at)
        .fetch_one(pool)
        .await
        .context("Failed to store clipboard entry")?;

        Ok(result)
    }

    /// Get a clipboard entry by key
    pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<ClipboardRecord>> {
        let now = Utc::now();

        // Update accessed_at timestamp
        sqlx::query("UPDATE clipboard SET accessed_at = ?1 WHERE key = ?2")
            .bind(now)
            .bind(key)
            .execute(pool)
            .await?;

        let record = sqlx::query_as::<_, ClipboardRecord>(
            "SELECT * FROM clipboard WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
        )
        .bind(key)
        .bind(now)
        .fetch_optional(pool)
        .await
        .context("Failed to get clipboard entry")?;

        Ok(record)
    }

    /// List all clipboard entries
    pub async fn list(pool: &SqlitePool) -> Result<Vec<ClipboardRecord>> {
        let now = Utc::now();
        let records = sqlx::query_as::<_, ClipboardRecord>(
            "SELECT * FROM clipboard WHERE expires_at IS NULL OR expires_at > ?1 ORDER BY updated_at DESC"
        )
        .bind(now)
        .fetch_all(pool)
        .await
        .context("Failed to list clipboard entries")?;

        Ok(records)
    }

    /// Delete a clipboard entry
    pub async fn delete(pool: &SqlitePool, key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM clipboard WHERE key = ?1")
            .bind(key)
            .execute(pool)
            .await
            .context("Failed to delete clipboard entry")?;

        Ok(result.rows_affected() > 0)
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(pool: &SqlitePool) -> Result<u64> {
        let now = Utc::now();
        let result =
            sqlx::query("DELETE FROM clipboard WHERE expires_at IS NOT NULL AND expires_at < ?1")
                .bind(now)
                .execute(pool)
                .await
                .context("Failed to cleanup expired clipboard entries")?;

        debug!(
            "Cleaned up {} expired clipboard entries",
            result.rows_affected()
        );
        Ok(result.rows_affected())
    }
}

/// Process history queries
pub struct ProcessHistoryQueries;

impl ProcessHistoryQueries {
    /// Record process start
    pub async fn record_start(
        pool: &SqlitePool,
        process_id: &str,
        name: &str,
        command: &str,
        args: Option<String>,
        env: Option<String>,
        cwd: Option<&str>,
    ) -> Result<i64> {
        let now = Utc::now();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO process_history (process_id, name, command, args, env, cwd, started_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING id
            "#
        )
        .bind(process_id)
        .bind(name)
        .bind(command)
        .bind(args)
        .bind(env)
        .bind(cwd)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .context("Failed to record process start")?;

        Ok(result)
    }

    /// Record process stop
    pub async fn record_stop(
        pool: &SqlitePool,
        process_id: &str,
        exit_code: Option<i32>,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE process_history
            SET stopped_at = ?1, exit_code = ?2, error = ?3
            WHERE process_id = ?4 AND stopped_at IS NULL
            "#,
        )
        .bind(now)
        .bind(exit_code)
        .bind(error)
        .bind(process_id)
        .execute(pool)
        .await
        .context("Failed to record process stop")?;

        Ok(())
    }

    /// Get process history
    pub async fn get_history(
        pool: &SqlitePool,
        process_id: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ProcessHistoryRecord>> {
        let mut query = "SELECT * FROM process_history".to_string();

        if process_id.is_some() {
            query.push_str(" WHERE process_id = ?1");
        }

        query.push_str(" ORDER BY started_at DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let records = if let Some(pid) = process_id {
            sqlx::query_as::<_, ProcessHistoryRecord>(&query)
                .bind(pid)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as::<_, ProcessHistoryRecord>(&query)
                .fetch_all(pool)
                .await?
        };

        Ok(records)
    }
}

/// Process queries (current configuration)
pub struct ProcessQueries;

impl ProcessQueries {
    /// Create or update a process
    pub async fn upsert(pool: &SqlitePool, process: &crate::types::ProcessInfo) -> Result<i64> {
        let args_json = serde_json::to_string(&process.args)?;
        let env_json = serde_json::to_string(&process.env)?;
        let tags_json = serde_json::to_string(&process.tags)?;
        let now = Utc::now();

        // Convert ProcessState to string
        let state = match process.status.state {
            crate::types::ProcessState::NotStarted => "not_started",
            crate::types::ProcessState::Running => "running",
            crate::types::ProcessState::Stopped => "stopped",
            crate::types::ProcessState::Failed => "failed",
        };

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO processes (
                process_id, name, command, args, env, cwd, state,
                pid, exit_code, started_at, stopped_at, error,
                tags, auto_start_on_restore, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(process_id) DO UPDATE SET
                name = excluded.name,
                command = excluded.command,
                args = excluded.args,
                env = excluded.env,
                cwd = excluded.cwd,
                state = excluded.state,
                pid = excluded.pid,
                exit_code = excluded.exit_code,
                started_at = excluded.started_at,
                stopped_at = excluded.stopped_at,
                error = excluded.error,
                tags = excluded.tags,
                auto_start_on_restore = excluded.auto_start_on_restore,
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )
        .bind(&process.process_id)
        .bind(&process.name)
        .bind(&process.command)
        .bind(args_json)
        .bind(env_json)
        .bind(&process.cwd)
        .bind(state)
        .bind(process.status.pid.map(|p| p as i32))
        .bind(process.status.exit_code)
        .bind(process.status.started_at)
        .bind(process.status.stopped_at)
        .bind(&process.status.error)
        .bind(tags_json)
        .bind(process.auto_start_on_restore)
        .bind(process.created_at)
        .bind(now)
        .fetch_one(pool)
        .await
        .context("Failed to upsert process")?;

        Ok(result)
    }

    /// Get a process by ID
    pub async fn get(pool: &SqlitePool, process_id: &str) -> Result<Option<ProcessRecord>> {
        let record =
            sqlx::query_as::<_, ProcessRecord>("SELECT * FROM processes WHERE process_id = ?1")
                .bind(process_id)
                .fetch_optional(pool)
                .await
                .context("Failed to get process")?;

        Ok(record)
    }

    /// List all processes
    pub async fn list(pool: &SqlitePool) -> Result<Vec<ProcessRecord>> {
        let records =
            sqlx::query_as::<_, ProcessRecord>("SELECT * FROM processes ORDER BY updated_at DESC")
                .fetch_all(pool)
                .await
                .context("Failed to list processes")?;

        Ok(records)
    }

    /// Delete a process
    pub async fn delete(pool: &SqlitePool, process_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM processes WHERE process_id = ?1")
            .bind(process_id)
            .execute(pool)
            .await
            .context("Failed to delete process")?;

        Ok(result.rows_affected() > 0)
    }
}

/// Process template queries
pub struct ProcessTemplateQueries;

impl ProcessTemplateQueries {
    /// Create or update a template
    pub async fn upsert(
        pool: &SqlitePool,
        template: &crate::types::ProcessTemplate,
    ) -> Result<i64> {
        let args_json = serde_json::to_string(&template.args)?;
        let env_json = serde_json::to_string(&template.env)?;
        let variables_json = serde_json::to_string(&template.variables)?;
        let tags_json = serde_json::to_string(&template.tags)?;
        let now = Utc::now();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO process_templates (
                template_id, name, description, category, command,
                args, env, default_cwd, default_auto_start,
                variables, tags, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(template_id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                category = excluded.category,
                command = excluded.command,
                args = excluded.args,
                env = excluded.env,
                default_cwd = excluded.default_cwd,
                default_auto_start = excluded.default_auto_start,
                variables = excluded.variables,
                tags = excluded.tags,
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )
        .bind(&template.template_id)
        .bind(&template.name)
        .bind(&template.description)
        .bind(&template.category)
        .bind(&template.command)
        .bind(args_json)
        .bind(env_json)
        .bind(&template.default_cwd)
        .bind(template.default_auto_start)
        .bind(variables_json)
        .bind(tags_json)
        .bind(template.created_at)
        .bind(now)
        .fetch_one(pool)
        .await
        .context("Failed to upsert template")?;

        Ok(result)
    }

    /// Get a template by ID
    pub async fn get(
        pool: &SqlitePool,
        template_id: &str,
    ) -> Result<Option<ProcessTemplateRecord>> {
        let record = sqlx::query_as::<_, ProcessTemplateRecord>(
            "SELECT * FROM process_templates WHERE template_id = ?1",
        )
        .bind(template_id)
        .fetch_optional(pool)
        .await
        .context("Failed to get template")?;

        Ok(record)
    }

    /// List all templates
    pub async fn list(pool: &SqlitePool) -> Result<Vec<ProcessTemplateRecord>> {
        let records = sqlx::query_as::<_, ProcessTemplateRecord>(
            "SELECT * FROM process_templates ORDER BY category, name",
        )
        .fetch_all(pool)
        .await
        .context("Failed to list templates")?;

        Ok(records)
    }

    /// Delete a template
    pub async fn delete(pool: &SqlitePool, template_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM process_templates WHERE template_id = ?1")
            .bind(template_id)
            .execute(pool)
            .await
            .context("Failed to delete template")?;

        Ok(result.rows_affected() > 0)
    }
}

/// Settings queries
pub struct SettingsQueries;

impl SettingsQueries {
    /// Set a setting value
    pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO settings (key, value, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .context("Failed to set setting")?;

        Ok(())
    }

    /// Get a setting value
    pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
        let record = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?1")
            .bind(key)
            .fetch_optional(pool)
            .await
            .context("Failed to get setting")?;

        Ok(record)
    }

    /// Get all settings
    pub async fn list(pool: &SqlitePool) -> Result<Vec<SettingsRecord>> {
        let records = sqlx::query_as::<_, SettingsRecord>("SELECT * FROM settings ORDER BY key")
            .fetch_all(pool)
            .await
            .context("Failed to list settings")?;

        Ok(records)
    }

    /// Delete a setting
    pub async fn delete(pool: &SqlitePool, key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM settings WHERE key = ?1")
            .bind(key)
            .execute(pool)
            .await
            .context("Failed to delete setting")?;

        Ok(result.rows_affected() > 0)
    }
}

/// System event queries
pub struct SystemEventQueries;

impl SystemEventQueries {
    /// Record a system event
    pub async fn record(
        pool: &SqlitePool,
        event_type: &str,
        description: &str,
        details: Option<String>,
        severity: &str,
    ) -> Result<i64> {
        let now = Utc::now();

        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO system_events (event_type, description, details, severity, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING id
            "#,
        )
        .bind(event_type)
        .bind(description)
        .bind(details)
        .bind(severity)
        .bind(now)
        .fetch_one(pool)
        .await
        .context("Failed to record system event")?;

        Ok(result)
    }

    /// Get system events
    pub async fn get_events(
        pool: &SqlitePool,
        event_type: Option<&str>,
        severity: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<SystemEventRecord>> {
        let mut query = "SELECT * FROM system_events WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(et) = event_type {
            query.push_str(" AND event_type = ?");
            params.push(et.to_string());
        }

        if let Some(sev) = severity {
            query.push_str(" AND severity = ?");
            params.push(sev.to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // Build query dynamically based on parameters
        let records = match (event_type, severity) {
            (Some(et), Some(sev)) => {
                sqlx::query_as::<_, SystemEventRecord>(&query)
                    .bind(et)
                    .bind(sev)
                    .fetch_all(pool)
                    .await?
            }
            (Some(et), None) => {
                sqlx::query_as::<_, SystemEventRecord>(&query)
                    .bind(et)
                    .fetch_all(pool)
                    .await?
            }
            (None, Some(sev)) => {
                sqlx::query_as::<_, SystemEventRecord>(&query)
                    .bind(sev)
                    .fetch_all(pool)
                    .await?
            }
            (None, None) => {
                sqlx::query_as::<_, SystemEventRecord>(&query)
                    .fetch_all(pool)
                    .await?
            }
        };

        Ok(records)
    }
}
