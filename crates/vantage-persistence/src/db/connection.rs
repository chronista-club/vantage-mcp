use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use tracing::{debug, info};

/// SurrealDB接続設定
///
/// # 環境変数
///
/// 以下の環境変数で設定を上書きできます：
/// - `VANTAGE_DB_ENDPOINT`: データベースエンドポイント（例: "127.0.0.1:8000"）
/// - `VANTAGE_DB_NAMESPACE`: 名前空間（デフォルト: "vantage"）
/// - `VANTAGE_DB_DATABASE`: データベース名（デフォルト: "main"）
/// - `VANTAGE_DB_USERNAME`: 認証ユーザー名（デフォルト: "vtg-local"）
/// - `VANTAGE_DB_PASSWORD`: 認証パスワード（デフォルト: "vtg-local"）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DbConfig {
    /// 環境変数から設定を読み込む
    ///
    /// 環境変数が設定されていない場合はデフォルト値を使用します。
    pub fn from_env() -> Self {
        let default = Self::default();

        Self {
            endpoint: std::env::var("VANTAGE_DB_ENDPOINT").unwrap_or(default.endpoint),
            namespace: std::env::var("VANTAGE_DB_NAMESPACE").unwrap_or(default.namespace),
            database: std::env::var("VANTAGE_DB_DATABASE").unwrap_or(default.database),
            username: std::env::var("VANTAGE_DB_USERNAME").unwrap_or(default.username),
            password: std::env::var("VANTAGE_DB_PASSWORD").unwrap_or(default.password),
        }
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:30300".to_string(),
            namespace: "vantage".to_string(),
            database: "main".to_string(),
            username: "vtg-local".to_string(),
            password: "vtg-local".to_string(),
        }
    }
}

/// SurrealDB接続ラッパー
///
/// WebSocket接続を使用してSurrealDBに接続します。
/// スレッドセーフで、複数のリポジトリから共有できます。
pub struct DbConnection {
    db: Surreal<Client>,
    config: DbConfig,
}

impl DbConnection {
    /// 新しい接続を作成
    ///
    /// # 引数
    ///
    /// * `config` - データベース接続設定
    ///
    /// # エラー
    ///
    /// 以下の場合にエラーを返します：
    /// - エンドポイントへの接続に失敗した場合
    /// - 認証に失敗した場合
    /// - 名前空間/データベースの選択に失敗した場合
    pub async fn new(config: DbConfig) -> Result<Self> {
        info!("Connecting to SurrealDB at {}", config.endpoint);

        let db = Surreal::new::<Ws>(&config.endpoint)
            .await
            .with_context(|| format!("Failed to connect to SurrealDB at {}", config.endpoint))?;

        debug!("Signing in with user: {}", config.username);
        db.signin(Root {
            username: &config.username,
            password: &config.password,
        })
        .await
        .with_context(|| format!("Failed to sign in as user: {}", config.username))?;

        debug!(
            "Using namespace: {}, database: {}",
            config.namespace, config.database
        );
        db.use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .with_context(|| {
                format!(
                    "Failed to use namespace '{}' and database '{}'",
                    config.namespace, config.database
                )
            })?;

        info!(
            "Successfully connected to SurrealDB ({}:{})",
            config.namespace, config.database
        );

        Ok(Self { db, config })
    }

    /// デフォルト設定で接続
    ///
    /// `DbConfig::default()`を使用して接続します。
    pub async fn new_default() -> Result<Self> {
        Self::new(DbConfig::default()).await
    }

    /// 環境変数から設定を読み込んで接続
    ///
    /// `DbConfig::from_env()`を使用して接続します。
    pub async fn new_from_env() -> Result<Self> {
        let config = DbConfig::from_env();
        Self::new(config).await
    }

    /// データベースハンドルを取得
    ///
    /// リポジトリパターンで使用するため、内部の`Surreal<Client>`への参照を返します。
    pub fn db(&self) -> &Surreal<Client> {
        &self.db
    }

    /// 設定を取得
    pub fn config(&self) -> &DbConfig {
        &self.config
    }

    /// 接続をテスト
    ///
    /// シンプルなクエリを実行して接続が有効かどうかを確認します。
    ///
    /// # エラー
    ///
    /// データベースへのクエリに失敗した場合にエラーを返します。
    pub async fn test_connection(&self) -> Result<()> {
        self.db
            .query("SELECT 1 AS test")
            .await
            .context("Failed to test database connection")?;

        debug!("Database connection test successful");
        Ok(())
    }

    /// 接続を正常にシャットダウン
    ///
    /// 明示的に接続をクローズする必要がある場合に使用します。
    /// 通常はDropで自動的に処理されます。
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down database connection");
        // SurrealDBのクライアントは自動的にクリーンアップされますが、
        // 将来的に明示的なシャットダウン処理が必要になった場合のための拡張ポイント
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // SurrealDBサーバーが起動している必要がある
    async fn test_connection() {
        let conn = DbConnection::new_default().await.unwrap();
        conn.test_connection().await.unwrap();
    }
}
