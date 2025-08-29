use facet::Facet;
use serde::{Deserialize, Serialize};

/// KDL形式のプロセス設定ファイルのルート構造
#[derive(Debug, Clone, Serialize, Deserialize, Facet)]
pub struct IchimiConfig {
    /// メタ情報（バージョンなど）
    #[facet(child)]
    pub meta: ConfigMeta,

    /// プロセス定義のリスト
    #[facet(child)]
    pub process: Vec<ProcessConfig>,
}

/// 設定ファイルのメタ情報
#[derive(Debug, Clone, Serialize, Deserialize, Facet)]
pub struct ConfigMeta {
    /// ファイルフォーマットのバージョン
    pub version: String,
}

/// プロセスの設定
#[derive(Debug, Clone, Serialize, Deserialize, Facet)]
pub struct ProcessConfig {
    /// プロセスID（ユニークな識別子）
    #[facet(argument)]
    pub id: String,

    /// 実行コマンド
    #[facet(property)]
    pub command: String,

    /// コマンドライン引数
    #[serde(default)]
    #[facet(property)]
    pub args: Vec<String>,

    /// 作業ディレクトリ（一時的に簡略化）
    #[serde(default)]
    #[facet(property)]
    pub cwd: String,

    // 環境変数（TODO: 後で実装）
    // #[serde(default)]
    // #[facet(child)]
    // pub env: HashMap<String, String>,
    /// 自動起動フラグ
    #[serde(default)]
    #[facet(property)]
    pub auto_start: bool,
}

impl Default for IchimiConfig {
    fn default() -> Self {
        Self {
            meta: ConfigMeta {
                version: "1.0.0".to_string(),
            },
            process: Vec::new(),
        }
    }
}

impl ProcessConfig {
    /// ProcessInfoから変換
    pub fn from_process_info(info: &crate::process::types::ProcessInfo) -> Self {
        Self {
            id: info.id.clone(),
            command: info.command.clone(),
            args: info.args.clone(),
            cwd: info
                .cwd
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            // env: info.env.clone(),
            auto_start: info.auto_start, // ProcessInfoからauto_startを取得
        }
    }

    /// ProcessInfoへ変換
    pub fn to_process_info(&self) -> crate::process::types::ProcessInfo {
        use std::collections::HashMap;
        use std::path::PathBuf;

        crate::process::types::ProcessInfo {
            id: self.id.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: HashMap::new(), // TODO: 環境変数サポート
            cwd: if self.cwd.is_empty() {
                None
            } else {
                Some(PathBuf::from(&self.cwd))
            },
            state: crate::process::types::ProcessState::NotStarted,
            auto_start: self.auto_start, // KDL設定からauto_startを取得
        }
    }
}

/// KDL形式のサンプルを生成
pub fn generate_sample_kdl() -> String {
    r#"// Ichimi Server Process Configuration
// Version: 1.0.0

meta {
    version "1.0.0"
    schema "ichimi-process-v1"
    created_at "2025-08-22T18:00:00Z"
}

// Example process definition
process "example-app" {
    command "/usr/bin/node"
    args "server.js" "--port" "3000"
    cwd "/var/www/app"
    
    env {
        NODE_ENV "production"
        PORT "3000"
    }
    
    auto_start false
}

// Another example
process "redis-server" {
    command "/usr/local/bin/redis-server"
    args "--port" "6379"
    auto_start true
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config_conversion() {
        use std::collections::HashMap;

        let info = crate::process::types::ProcessInfo {
            id: "test".to_string(),
            command: "/bin/echo".to_string(),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
            cwd: None,
            state: crate::process::types::ProcessState::NotStarted,
            auto_start: false,
        };

        let config = ProcessConfig::from_process_info(&info);
        assert_eq!(config.id, "test");
        assert_eq!(config.command, "/bin/echo");
        assert_eq!(config.args, vec!["hello"]);

        let info2 = config.to_process_info();
        assert_eq!(info2.id, info.id);
        assert_eq!(info2.command, info.command);
    }
}
