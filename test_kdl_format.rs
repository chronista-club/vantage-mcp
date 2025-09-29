use std::collections::HashMap;
use chrono::Utc;

fn main() {
    // Test KDL format generation
    let mut env = HashMap::new();
    env.insert("PORT".to_string(), "8000".to_string());
    env.insert("DEBUG".to_string(), "true".to_string());

    let kdl = generate_kdl_sample(env);
    println!("{}", kdl);

    // Test the format we want
    println!("\n=== Expected KDL Format ===\n");
    println!(r#"// Ichimi Process Snapshot
// 生成日時: {}

meta {{
    version "1.0"
    timestamp "{}"
    hostname "test-server"
}}

// Web Server (自動起動)
process "web-server" {{
    name "Web Server"
    command "python"
    args "-m http.server"
    cwd "/var/www"
    auto_start #true
    tag "web"

    // 環境変数
    env {{
        var "PORT" "8000"
        var "DEBUG" "true"
    }}

    // 実行中
    state "running" {{
        pid 12345
        started_at "{}"
    }}
}}
"#, Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), Utc::now().to_rfc3339());
}

fn generate_kdl_sample(env: HashMap<String, String>) -> String {
    let mut kdl = String::new();

    // Header
    kdl.push_str("// Ichimi Process Snapshot\n");
    kdl.push_str(&format!("// 生成日時: {}\n\n", Utc::now().to_rfc3339()));

    // Meta
    kdl.push_str("meta {\n");
    kdl.push_str("    version \"1.0\"\n");
    kdl.push_str(&format!("    timestamp \"{}\"\n", Utc::now().to_rfc3339()));
    kdl.push_str("    hostname \"test-server\"\n");
    kdl.push_str("}\n\n");

    // Process
    kdl.push_str("// Web Server (自動起動)\n");
    kdl.push_str("process \"web-server\" {\n");
    kdl.push_str("    name \"Web Server\"\n");
    kdl.push_str("    command \"python\"\n");
    kdl.push_str("    args \"-m http.server\"\n");
    kdl.push_str("    cwd \"/var/www\"\n");
    kdl.push_str("    auto_start #true\n");
    kdl.push_str("    tag \"web\"\n");

    // Environment
    kdl.push_str("\n    // 環境変数\n");
    kdl.push_str("    env {\n");
    for (key, value) in &env {
        kdl.push_str(&format!("        var \"{}\" \"{}\"\n", key, value));
    }
    kdl.push_str("    }\n");

    // State
    kdl.push_str("\n    // 実行中\n");
    kdl.push_str("    state \"running\" {\n");
    kdl.push_str("        pid 12345\n");
    kdl.push_str(&format!("        started_at \"{}\"\n", Utc::now().to_rfc3339()));
    kdl.push_str("    }\n");

    kdl.push_str("}\n");

    kdl
}