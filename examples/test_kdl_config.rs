use std::fs;
use facet_kdl;
use ichimi_server::persistence::kdl_schema::IchimiConfig;

fn main() -> anyhow::Result<()> {
    // ファイルパスを引数から取得、デフォルトは examples/ichimi.kdl
    let file_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/ichimi.kdl".to_string());
    
    // KDLファイルを読み込み
    let kdl_content = fs::read_to_string(&file_path)?;
    println!("読み込んだKDLファイル:");
    println!("{}", kdl_content);
    println!("\n===========================\n");
    
    // KDLをパース
    match facet_kdl::from_str::<IchimiConfig>(&kdl_content) {
        Ok(config) => {
            println!("✅ パース成功！");
            println!("\nメタ情報:");
            println!("  バージョン: {}", config.meta.version);
            
            println!("\nプロセス定義:");
            for process in &config.process {
                println!("\n  プロセスID: {}", process.id);
                println!("    コマンド: {}", process.command);
                println!("    引数: {:?}", process.args);
                println!("    作業ディレクトリ: {:?}", process.cwd);
                println!("    自動起動: {}", process.auto_start);
                // if !process.env.is_empty() {
                //     println!("    環境変数:");
                //     for (key, value) in &process.env {
                //         println!("      {} = {}", key, value);
                //     }
                // }
            }
        }
        Err(e) => {
            println!("❌ パースエラー: {:?}", e);
            return Err(anyhow::anyhow!("KDLパースに失敗しました: {:?}", e));
        }
    }
    
    Ok(())
}