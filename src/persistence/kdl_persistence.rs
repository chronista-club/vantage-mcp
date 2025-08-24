use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use super::kdl_schema::{IchimiConfig, ProcessConfig};

/// KDL形式での永続化を扱うモジュール
pub struct KdlPersistence {
    config_path: PathBuf,
}

impl KdlPersistence {
    /// 新しいKdlPersistenceインスタンスを作成
    pub fn new(config_dir: &Path) -> Self {
        let config_path = config_dir.join("processes.kdl");
        Self { config_path }
    }

    /// KDL設定ファイルからプロセス設定を読み込む
    pub fn load_config(&self) -> Result<IchimiConfig> {
        if !self.config_path.exists() {
            // ファイルが存在しない場合は空の設定を返す
            return Ok(IchimiConfig::default());
        }

        let kdl_content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read KDL file: {:?}", self.config_path))?;

        // facet-kdlを使用してデシリアライズ
        let config = facet_kdl::from_str(&kdl_content)
            .with_context(|| "Failed to parse KDL configuration")?;

        Ok(config)
    }

    /// プロセス設定をKDL形式で保存
    pub fn save_config(&self, config: &IchimiConfig) -> Result<()> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // 現時点ではKDLのシリアライズは手動で行う
        let kdl_content = self.generate_kdl(config);
        
        // アトミックな書き込みのため、一時ファイルを使用
        let temp_path = self.config_path.with_extension("kdl.tmp");
        fs::write(&temp_path, kdl_content)
            .with_context(|| format!("Failed to write to temp file: {:?}", temp_path))?;

        fs::rename(&temp_path, &self.config_path)
            .with_context(|| format!("Failed to rename temp file to: {:?}", self.config_path))?;

        Ok(())
    }

    /// IchimiConfigからKDL文字列を生成（手動実装）
    fn generate_kdl(&self, config: &IchimiConfig) -> String {
        let mut kdl = String::new();
        
        // ヘッダーコメント
        kdl.push_str("// Ichimi Server Process Configuration\n");
        kdl.push_str("// Auto-generated file - modifications will be preserved\n\n");

        // メタ情報
        kdl.push_str("meta {\n");
        kdl.push_str(&format!("    version \"{}\"\n", config.meta.version));
        if let Some(schema) = &config.meta.schema {
            kdl.push_str(&format!("    schema \"{}\"\n", schema));
        }
        if let Some(created_at) = &config.meta.created_at {
            kdl.push_str(&format!("    created_at \"{}\"\n", created_at));
        }
        if let Some(updated_at) = &config.meta.updated_at {
            kdl.push_str(&format!("    updated_at \"{}\"\n", updated_at));
        }
        kdl.push_str("}\n\n");

        // プロセス定義
        for process in &config.process {
            kdl.push_str(&format!("process \"{}\" {{\n", process.id));
            kdl.push_str(&format!("    command \"{}\"\n", process.command));
            
            if !process.args.is_empty() {
                kdl.push_str("    args");
                for arg in &process.args {
                    kdl.push_str(&format!(" \"{}\"", arg));
                }
                kdl.push('\n');
            }
            
            if let Some(cwd) = &process.cwd {
                kdl.push_str(&format!("    cwd \"{}\"\n", cwd.display()));
            }
            
            if !process.env.is_empty() {
                kdl.push_str("    env {\n");
                for (key, value) in &process.env {
                    kdl.push_str(&format!("        {} \"{}\"\n", key, value));
                }
                kdl.push_str("    }\n");
            }
            
            kdl.push_str(&format!("    auto_start {}\n", process.auto_start));
            kdl.push_str("}\n\n");
        }

        kdl
    }

    /// プロセス設定を追加または更新
    pub fn add_or_update_process(&self, process: ProcessConfig) -> Result<()> {
        let mut config = self.load_config()?;
        
        // 既存のプロセスを更新するか、新規追加
        if let Some(existing) = config.process.iter_mut().find(|p| p.id == process.id) {
            *existing = process;
        } else {
            config.process.push(process);
        }

        // 更新日時を設定
        config.meta.updated_at = Some(chrono::Utc::now().to_rfc3339());

        self.save_config(&config)?;
        Ok(())
    }

    /// プロセス設定を削除
    pub fn remove_process(&self, process_id: &str) -> Result<bool> {
        let mut config = self.load_config()?;
        let initial_len = config.process.len();
        
        config.process.retain(|p| p.id != process_id);
        
        if config.process.len() < initial_len {
            // 更新日時を設定
            config.meta.updated_at = Some(chrono::Utc::now().to_rfc3339());
            self.save_config(&config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// すべてのプロセス設定を取得
    pub fn get_all_processes(&self) -> Result<Vec<ProcessConfig>> {
        let config = self.load_config()?;
        Ok(config.process)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = KdlPersistence::new(temp_dir.path());

        let process = ProcessConfig {
            id: "test-process".to_string(),
            command: "/usr/bin/test".to_string(),
            args: vec!["arg1".to_string(), "arg2".to_string()],
            cwd: Some(PathBuf::from("/tmp")),
            env: [("KEY".to_string(), "VALUE".to_string())].into(),
            auto_start: true,
        };

        // プロセスを追加
        persistence.add_or_update_process(process.clone()).unwrap();
        
        // デバッグ用: KDLファイルの内容を表示
        let kdl_path = temp_dir.path().join("processes.kdl");
        if kdl_path.exists() {
            let content = std::fs::read_to_string(&kdl_path).unwrap();
            println!("Generated KDL content:\n{}", content);
        }

        // プロセスを読み込み
        let processes = persistence.get_all_processes().unwrap();
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].id, "test-process");
        assert_eq!(processes[0].command, "/usr/bin/test");
    }

    #[test]
    fn test_remove_process() {
        let temp_dir = TempDir::new().unwrap();
        let persistence = KdlPersistence::new(temp_dir.path());

        let process = ProcessConfig {
            id: "test-process".to_string(),
            command: "/usr/bin/test".to_string(),
            args: vec![],
            cwd: None,
            env: HashMap::new(),
            auto_start: false,
        };

        // プロセスを追加
        persistence.add_or_update_process(process).unwrap();
        assert_eq!(persistence.get_all_processes().unwrap().len(), 1);

        // プロセスを削除
        let removed = persistence.remove_process("test-process").unwrap();
        assert!(removed);
        assert_eq!(persistence.get_all_processes().unwrap().len(), 0);
    }
}