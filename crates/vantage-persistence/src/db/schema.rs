use anyhow::{Context, Result};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tracing::{debug, info};

/// スキーママネージャー
///
/// データベーススキーマの初期化と管理を行います。
/// スキーマファイル(.surql)はコンパイル時に埋め込まれ、
/// IDEMPOTENTな方法で適用されます（OVERWRITE戦略を使用）。
pub struct SchemaManager<'a> {
    db: &'a Surreal<Client>,
}

impl<'a> SchemaManager<'a> {
    /// 新しいスキーママネージャーを作成
    ///
    /// # 引数
    ///
    /// * `db` - SurrealDBクライアントへの参照
    pub fn new(db: &'a Surreal<Client>) -> Self {
        Self { db }
    }

    /// 全スキーマを適用
    ///
    /// テーブル定義とインデックスを順番に適用します。
    /// OVERWRITE戦略を使用しているため、何度実行しても安全です。
    ///
    /// # エラー
    ///
    /// スキーマの適用に失敗した場合にエラーを返します。
    pub async fn apply_all(&self) -> Result<()> {
        info!("Applying all schemas");

        self.apply_tables().await?;
        self.apply_indexes().await?;

        info!("All schemas applied successfully");
        Ok(())
    }

    /// テーブルスキーマを適用
    ///
    /// 01_tables/以下の全テーブル定義を適用します。
    async fn apply_tables(&self) -> Result<()> {
        debug!("Applying table schemas");

        // テンプレートテーブル
        let template_schema = include_str!("../../schema/01_tables/template.surql");
        self.execute_schema(template_schema, "template table").await?;

        Ok(())
    }

    /// インデックスを適用
    ///
    /// 02_indexes/以下の全インデックス定義を適用します。
    /// テーブルが存在している必要があるため、apply_tables()の後に実行されます。
    async fn apply_indexes(&self) -> Result<()> {
        debug!("Applying indexes");

        // テンプレートインデックス
        let template_indexes = include_str!("../../schema/02_indexes/template_indexes.surql");
        self.execute_schema(template_indexes, "template indexes").await?;

        Ok(())
    }

    /// スキーマSQLを実行
    ///
    /// # 引数
    ///
    /// * `sql` - 実行するSurrealQLスキーマ定義
    /// * `description` - ログ出力用の説明文
    ///
    /// # エラー
    ///
    /// クエリ実行に失敗した場合にエラーを返します。
    async fn execute_schema(&self, sql: &str, description: &str) -> Result<()> {
        debug!("Executing schema: {}", description);

        self.db
            .query(sql)
            .await
            .with_context(|| format!("Failed to execute schema: {}", description))?;

        debug!("Successfully executed: {}", description);
        Ok(())
    }

    /// スキーマバージョンを確認（将来の拡張用）
    ///
    /// マイグレーション管理を実装する際に使用します。
    /// 現在は未実装で、常にNoneを返します。
    #[allow(dead_code)]
    pub async fn check_version(&self) -> Result<Option<String>> {
        // TODO: schema_versionテーブルを実装してバージョン管理を行う
        // CREATE TABLE schema_version SCHEMAFULL;
        // DEFINE FIELD version ON TABLE schema_version TYPE string;
        // DEFINE FIELD applied_at ON TABLE schema_version TYPE datetime;
        Ok(None)
    }

    /// スキーマをリセット（開発・テスト用）
    ///
    /// **警告**: 全データが削除されます。本番環境では使用しないでください。
    #[allow(dead_code)]
    pub async fn reset_all(&self) -> Result<()> {
        info!("Resetting all schemas (WARNING: all data will be lost)");

        // テーブルを削除
        self.db
            .query("REMOVE TABLE template")
            .await
            .context("Failed to remove template table")?;

        info!("All schemas reset successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbConnection;

    #[tokio::test]
    #[ignore] // SurrealDBサーバーが起動している必要がある
    async fn test_apply_schema() {
        let conn = DbConnection::new_default().await.unwrap();
        let schema_manager = SchemaManager::new(conn.db());

        schema_manager.apply_all().await.unwrap();
    }
}
