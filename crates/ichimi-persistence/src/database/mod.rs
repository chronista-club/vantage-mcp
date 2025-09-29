use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info};

pub mod migrations;
pub mod models;
pub mod queries;

/// SQLite database connection manager
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        let database_url = format!("sqlite://{}", database_path.display());
        debug!("Connecting to database: {}", database_url);

        // Configure connection options
        let options = SqliteConnectOptions::from_str(&database_url)?
            .create_if_missing(true)
            .pragma("journal_mode", "WAL")
            .pragma("foreign_keys", "ON")
            .pragma("busy_timeout", "5000")
            .pragma("synchronous", "NORMAL");

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to create database pool")?;

        info!("Database connection established");

        let db = Self { pool };
        
        // Run migrations
        db.run_migrations().await?;
        
        Ok(db)
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        // Execute migration scripts
        let migrations = [
            include_str!("../../migrations/20241229_000001_initial_schema.sql"),
            include_str!("../../migrations/20241229_000002_processes_and_templates.sql"),
        ];

        for (i, migration) in migrations.iter().enumerate() {
            debug!("Running migration {}...", i + 1);
            sqlx::raw_sql(migration)
                .execute(&self.pool)
                .await
                .context(format!("Failed to run migration {}", i + 1))?;
        }

        info!("Database migrations completed");
        Ok(())
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .context("Failed to test database connection")?;
        
        Ok(())
    }

    /// Close database connections
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path).await.unwrap();
        assert!(db.test_connection().await.is_ok());
        
        db.close().await.unwrap();
    }
}
