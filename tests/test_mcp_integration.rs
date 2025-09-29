// MCP統合テスト - ichimiがichimi自身を管理する

use std::process::{Command, Stdio};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use std::thread;
use serde_json::json;

#[test]
#[ignore] // 手動実行用
fn test_ichimi_self_hosting() {
    // Ichimiサーバーを起動
    let mut ichimi = Command::new("./target/release/ichimi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ichimi server");

    let mut stdin = ichimi.stdin.take().expect("Failed to get stdin");
    let stdout = ichimi.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);

    // 初期化リクエスト
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            }
        },
        "id": 1
    });

    stdin.write_all(format!("{}\n", init_request).as_bytes())
        .expect("Failed to write init request");
    stdin.flush().expect("Failed to flush stdin");

    thread::sleep(Duration::from_secs(1));

    // get_statusテスト
    let status_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "get_status",
            "arguments": {}
        },
        "id": 2
    });

    stdin.write_all(format!("{}\n", status_request).as_bytes())
        .expect("Failed to write status request");
    stdin.flush().expect("Failed to flush stdin");

    // レスポンスを読み取り
    let mut lines = reader.lines();
    let mut response_count = 0;

    for line in lines {
        if let Ok(line) = line {
            println!("Response: {}", line);
            response_count += 1;
            if response_count >= 2 {
                break;
            }
        }
    }

    // プロセス作成テスト
    let create_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "create_process",
            "arguments": {
                "name": "test-process",
                "command": "echo",
                "args": ["Hello from ichimi self-test"],
                "env": {},
                "cwd": "."
            }
        },
        "id": 3
    });

    stdin.write_all(format!("{}\n", create_request).as_bytes())
        .expect("Failed to write create request");
    stdin.flush().expect("Failed to flush stdin");

    thread::sleep(Duration::from_secs(1));

    // サーバーを終了
    drop(stdin); // stdinを閉じる
    ichimi.kill().expect("Failed to kill ichimi");

    assert!(response_count > 0, "No responses received from ichimi");
}

#[test]
#[ignore]
fn test_process_lifecycle() {
    // より詳細なプロセスライフサイクルテスト
    let test_script = r#"
#!/bin/bash
echo "Test process started"
trap 'echo "Received SIGTERM"; exit 0' SIGTERM
while true; do
    echo "Running..."
    sleep 1
done
"#;

    // テストスクリプトを作成
    std::fs::write("/tmp/test_lifecycle.sh", test_script)
        .expect("Failed to create test script");

    std::process::Command::new("chmod")
        .args(&["+x", "/tmp/test_lifecycle.sh"])
        .output()
        .expect("Failed to make script executable");

    // Ichimiサーバーを起動
    let mut ichimi = Command::new("./target/release/ichimi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ichimi server");

    let mut stdin = ichimi.stdin.take().expect("Failed to get stdin");

    // プロセスを作成
    let create_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "create_process",
            "arguments": {
                "name": "lifecycle-test",
                "command": "/tmp/test_lifecycle.sh",
                "args": [],
                "env": {},
                "cwd": "/tmp"
            }
        },
        "id": 1
    });

    stdin.write_all(format!("{}\n", create_request).as_bytes())
        .expect("Failed to write create request");

    // プロセスを開始
    let start_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "start_process",
            "arguments": {
                "id": "lifecycle-test"
            }
        },
        "id": 2
    });

    stdin.write_all(format!("{}\n", start_request).as_bytes())
        .expect("Failed to write start request");

    thread::sleep(Duration::from_secs(2));

    // プロセスを停止
    let stop_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "stop_process",
            "arguments": {
                "id": "lifecycle-test"
            }
        },
        "id": 3
    });

    stdin.write_all(format!("{}\n", stop_request).as_bytes())
        .expect("Failed to write stop request");

    thread::sleep(Duration::from_secs(1));

    // クリーンアップ
    drop(stdin);
    ichimi.kill().expect("Failed to kill ichimi");
    std::fs::remove_file("/tmp/test_lifecycle.sh").ok();
}

#[test]
#[ignore]
fn test_multiple_processes_stress() {
    // 複数プロセスの同時管理ストレステスト
    let mut ichimi = Command::new("./target/release/ichimi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ichimi server");

    let mut stdin = ichimi.stdin.take().expect("Failed to get stdin");

    // 10個のプロセスを作成
    for i in 1..=10 {
        let create_request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "create_process",
                "arguments": {
                    "name": format!("stress-test-{}", i),
                    "command": "sleep",
                    "args": ["30"],
                    "env": {},
                    "cwd": "."
                }
            },
            "id": i
        });

        stdin.write_all(format!("{}\n", create_request).as_bytes())
            .expect("Failed to write create request");
    }

    // 全プロセスを起動
    for i in 1..=10 {
        let start_request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "start_process",
                "arguments": {
                    "id": format!("stress-test-{}", i)
                }
            },
            "id": 100 + i
        });

        stdin.write_all(format!("{}\n", start_request).as_bytes())
            .expect("Failed to write start request");

        thread::sleep(Duration::from_millis(100));
    }

    // プロセス一覧を取得
    let list_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "list_processes",
            "arguments": {}
        },
        "id": 200
    });

    stdin.write_all(format!("{}\n", list_request).as_bytes())
        .expect("Failed to write list request");

    thread::sleep(Duration::from_secs(2));

    // 全プロセスを停止
    for i in 1..=10 {
        let stop_request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "stop_process",
                "arguments": {
                    "id": format!("stress-test-{}", i)
                }
            },
            "id": 300 + i
        });

        stdin.write_all(format!("{}\n", stop_request).as_bytes())
            .expect("Failed to write stop request");
    }

    thread::sleep(Duration::from_secs(1));

    // クリーンアップ
    drop(stdin);
    ichimi.kill().expect("Failed to kill ichimi");
}

#[test]
#[ignore]
fn test_auto_restart_on_failure() {
    // プロセスが失敗した場合の自動再起動テスト
    let failing_script = r#"
#!/bin/bash
echo "Process started at $(date)"
sleep 2
echo "Simulating failure..."
exit 1
"#;

    std::fs::write("/tmp/failing_process.sh", failing_script)
        .expect("Failed to create failing script");

    std::process::Command::new("chmod")
        .args(&["+x", "/tmp/failing_process.sh"])
        .output()
        .expect("Failed to make script executable");

    let mut ichimi = Command::new("./target/release/ichimi")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ichimi server");

    let mut stdin = ichimi.stdin.take().expect("Failed to get stdin");

    // auto_start_on_restoreフラグ付きでプロセスを作成
    let create_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "create_process",
            "arguments": {
                "name": "auto-restart-test",
                "command": "/tmp/failing_process.sh",
                "args": [],
                "env": {},
                "cwd": "/tmp",
                "auto_start_on_restore": true
            }
        },
        "id": 1
    });

    stdin.write_all(format!("{}\n", create_request).as_bytes())
        .expect("Failed to write create request");

    // プロセスを開始
    let start_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "start_process",
            "arguments": {
                "id": "auto-restart-test"
            }
        },
        "id": 2
    });

    stdin.write_all(format!("{}\n", start_request).as_bytes())
        .expect("Failed to write start request");

    // プロセスが失敗するのを待つ
    thread::sleep(Duration::from_secs(3));

    // ステータスを確認
    let status_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "get_process_status",
            "arguments": {
                "id": "auto-restart-test"
            }
        },
        "id": 3
    });

    stdin.write_all(format!("{}\n", status_request).as_bytes())
        .expect("Failed to write status request");

    thread::sleep(Duration::from_secs(1));

    // クリーンアップ
    drop(stdin);
    ichimi.kill().expect("Failed to kill ichimi");
    std::fs::remove_file("/tmp/failing_process.sh").ok();
}