#!/bin/bash

# IchimiのMCPツールを使用したセルフテスト
# ichimiサーバーがichimi自体でプロセスを管理する

set -e

echo "=================================================="
echo "Ichimi Self-Test using MCP Tools"
echo "=================================================="

# プロジェクトルート
PROJECT_DIR=$(pwd)
echo "Project directory: $PROJECT_DIR"

# テスト用ディレクトリ
TEST_DIR="$PROJECT_DIR/.mcp_test"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# テストプロセススクリプトの作成
cat > "$TEST_DIR/test_process.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] Test process started with PID $$"
trap 'echo "[$(date +%H:%M:%S)] Received SIGTERM, shutting down gracefully..."; exit 0' SIGTERM
while true; do
    echo "[$(date +%H:%M:%S)] Process running..."
    sleep 2
done
EOF
chmod +x "$TEST_DIR/test_process.sh"

# MCPクライアントのPythonスクリプトを作成
cat > "$TEST_DIR/test_mcp_client.py" << 'EOF'
#!/usr/bin/env python3
import json
import subprocess
import sys
import time

def send_mcp_request(method, params=None):
    """Send MCP request to ichimi server"""
    request = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params or {},
        "id": 1
    }

    # ichimiサーバーはstdin/stdoutを使用
    proc = subprocess.Popen(
        ["./target/release/ichimi", "--no-web"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # リクエストを送信
    proc.stdin.write(json.dumps(request) + "\n")
    proc.stdin.flush()

    # レスポンスを読み取り（タイムアウト付き）
    response = proc.stdout.readline()
    proc.terminate()

    if response:
        return json.loads(response)
    return None

# Test 1: Create process
print("1. Creating test process...")
response = send_mcp_request("tools/call", {
    "name": "create_process",
    "arguments": {
        "name": "test-process",
        "command": sys.argv[1] + "/test_process.sh",
        "args": [],
        "env": {},
        "cwd": sys.argv[1]
    }
})
print(f"Response: {response}")

# Test 2: List processes
print("\n2. Listing processes...")
response = send_mcp_request("tools/call", {
    "name": "list_processes",
    "arguments": {}
})
print(f"Response: {response}")

# Test 3: Get status
print("\n3. Getting status...")
response = send_mcp_request("tools/call", {
    "name": "get_status",
    "arguments": {}
})
print(f"Response: {response}")
EOF

# Node.jsを使用したMCPテストクライアント
cat > "$TEST_DIR/test_mcp_client.js" << 'EOF'
const { spawn } = require('child_process');

// MCP request helper
function sendMcpRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
        const request = {
            jsonrpc: "2.0",
            method: method,
            params: params,
            id: Date.now()
        };

        const ichimi = spawn('./target/release/ichimi', ['--no-web'], {
            stdio: ['pipe', 'pipe', 'pipe']
        });

        let responseData = '';
        let errorData = '';

        ichimi.stdout.on('data', (data) => {
            responseData += data.toString();
        });

        ichimi.stderr.on('data', (data) => {
            errorData += data.toString();
        });

        ichimi.on('close', () => {
            if (responseData) {
                try {
                    // MCPレスポンスは複数行の可能性がある
                    const lines = responseData.split('\n').filter(line => line.trim());
                    const lastLine = lines[lines.length - 1];
                    const response = JSON.parse(lastLine);
                    resolve(response);
                } catch (e) {
                    reject(new Error(`Parse error: ${e.message}\nResponse: ${responseData}`));
                }
            } else {
                reject(new Error(`No response. Error: ${errorData}`));
            }
        });

        // Send request
        ichimi.stdin.write(JSON.stringify(request) + '\n');
        ichimi.stdin.end();
    });
}

async function runTests() {
    const testDir = process.argv[2] || '.mcp_test';

    try {
        console.log('=== Ichimi MCP Self-Test ===\n');

        // Test 1: Get server status
        console.log('1. Getting server status...');
        const status = await sendMcpRequest('tools/call', {
            name: 'get_status',
            arguments: {}
        });
        console.log('Status:', JSON.stringify(status, null, 2));

        // Test 2: Create a process
        console.log('\n2. Creating test process...');
        const createResult = await sendMcpRequest('tools/call', {
            name: 'create_process',
            arguments: {
                name: 'mcp-test-process',
                command: `${testDir}/test_process.sh`,
                args: [],
                env: {},
                cwd: testDir
            }
        });
        console.log('Create result:', JSON.stringify(createResult, null, 2));

        // Test 3: List processes
        console.log('\n3. Listing processes...');
        const processes = await sendMcpRequest('tools/call', {
            name: 'list_processes',
            arguments: {}
        });
        console.log('Processes:', JSON.stringify(processes, null, 2));

    } catch (error) {
        console.error('Test failed:', error);
        process.exit(1);
    }
}

runTests();
EOF

echo ""
echo "1. Building Ichimi if needed..."
if [ ! -f "./target/release/ichimi" ]; then
    cargo build --release --bin ichimi
fi

echo ""
echo "2. Testing MCP communication directly..."

# MCPプロトコルで直接通信のテスト
echo ""
echo "3. Sending test MCP requests..."

# JSONでMCPリクエストを作成
cat > "$TEST_DIR/test_requests.jsonl" << EOF
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_status","arguments":{}},"id":2}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"create_process","arguments":{"name":"test-proc","command":"$TEST_DIR/test_process.sh","args":[],"env":{},"cwd":"$TEST_DIR"}},"id":3}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_processes","arguments":{}},"id":4}
EOF

echo "Sending MCP requests to Ichimi server..."
echo "Request 1: Initialize"
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}}},"id":1}' | ./target/release/ichimi --no-web 2>/dev/null | head -5 || echo "No response"

echo ""
echo "4. Testing process management via HTTP API..."
# HTTPエンドポイント経由でのテスト（web-onlyモードが起動している場合）
if curl -s http://localhost:12701/api/status > /dev/null 2>&1; then
    echo "✓ Web server is running on port 12701"

    # プロセスを作成
    echo ""
    echo "Creating process via HTTP API..."
    curl -X POST http://localhost:12701/api/processes \
         -H "Content-Type: application/json" \
         -d "{
           \"name\": \"http-test-process\",
           \"command\": \"$TEST_DIR/test_process.sh\",
           \"args\": [],
           \"env\": {},
           \"cwd\": \"$TEST_DIR\"
         }" 2>/dev/null | jq . || echo "Failed to create process"

    # プロセス一覧を取得
    echo ""
    echo "Listing processes via HTTP API..."
    curl -s http://localhost:12701/api/processes | jq . || echo "Failed to list processes"

    # ステータスを取得
    echo ""
    echo "Getting status via HTTP API..."
    curl -s http://localhost:12701/api/status | jq . || echo "Failed to get status"
else
    echo "✗ Web server is not running on port 12701"
fi

echo ""
echo "5. Testing graceful shutdown behavior..."

# プロセスを起動してシャットダウンをテスト
if [ -f "$TEST_DIR/test_process.sh" ]; then
    echo "Starting test process directly..."
    "$TEST_DIR/test_process.sh" > "$TEST_DIR/process.log" 2>&1 &
    TEST_PID=$!
    echo "Process started with PID $TEST_PID"

    sleep 2
    echo "Sending SIGTERM..."
    kill -TERM $TEST_PID

    # グレースフルシャットダウンの確認
    for i in {1..5}; do
        if ! ps -p $TEST_PID > /dev/null 2>&1; then
            echo "✓ Process shut down gracefully after $i second(s)"
            break
        fi
        sleep 1
    done

    # ログ確認
    if grep -q "SIGTERM" "$TEST_DIR/process.log"; then
        echo "✓ Process handled SIGTERM correctly"
        cat "$TEST_DIR/process.log"
    fi
fi

echo ""
echo "=================================================="
echo "Test Summary"
echo "=================================================="

# テスト結果のサマリー
echo "1. MCP Protocol: Testing direct MCP communication"
echo "2. HTTP API: Testing via web interface"
echo "3. Process Management: Testing process lifecycle"
echo "4. Graceful Shutdown: Testing SIGTERM handling"

# クリーンアップ
echo ""
echo "Cleaning up..."
pkill -f test_process.sh 2>/dev/null || true
# rm -rf "$TEST_DIR"

echo ""
echo "✓ Self-test completed!"