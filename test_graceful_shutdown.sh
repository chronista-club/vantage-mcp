#!/bin/bash
# Test script for graceful shutdown functionality

set -e

echo "=== Ichimi Server Graceful Shutdown Test ==="

# カラー出力の設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# テスト用のPythonスクリプトを作成（SIGTERMを適切に処理する）
cat > /tmp/graceful_test.py << 'EOF'
#!/usr/bin/env python3
import signal
import sys
import time
import os

def signal_handler(signum, frame):
    print(f"Received signal {signum}, performing graceful shutdown...", flush=True)
    # クリーンアップ処理をシミュレート
    for i in range(3):
        print(f"Cleanup step {i+1}/3...", flush=True)
        time.sleep(0.5)
    print("Graceful shutdown complete", flush=True)
    sys.exit(0)

# SIGTERMハンドラを設定
signal.signal(signal.SIGTERM, signal_handler)

print(f"Process started with PID {os.getpid()}", flush=True)
print("Running... (press Ctrl+C or send SIGTERM to stop gracefully)", flush=True)

try:
    while True:
        print("Working...", flush=True)
        time.sleep(2)
except KeyboardInterrupt:
    print("Received KeyboardInterrupt, exiting...", flush=True)
    sys.exit(0)
EOF

chmod +x /tmp/graceful_test.py

# テスト用のスクリプト（SIGTERMを無視する）
cat > /tmp/stubborn_test.py << 'EOF'
#!/usr/bin/env python3
import signal
import sys
import time
import os

def signal_handler(signum, frame):
    print(f"Received signal {signum}, but I'm stubborn and won't exit!", flush=True)
    # SIGTERMを無視

# SIGTERMハンドラを設定（無視する）
signal.signal(signal.SIGTERM, signal_handler)

print(f"Stubborn process started with PID {os.getpid()}", flush=True)
print("I will ignore SIGTERM signals!", flush=True)

try:
    while True:
        print("I'm still running stubbornly...", flush=True)
        time.sleep(2)
except KeyboardInterrupt:
    print("OK, KeyboardInterrupt, I'll exit...", flush=True)
    sys.exit(0)
EOF

chmod +x /tmp/stubborn_test.py

# ビルド
echo -e "\n${YELLOW}Building Ichimi Server...${NC}"
cargo build --release

# Ichimi ServerをMCPサーバーとして起動
echo -e "\n${GREEN}Starting Ichimi Server...${NC}"
RUST_LOG=info ./target/release/ichimi > /tmp/ichimi_test.log 2>&1 &
ICHIMI_PID=$!

# サーバーが起動するまで待つ
sleep 2

# MCPコマンドを送信するヘルパー関数
send_mcp_command() {
    local method=$1
    local params=$2
    echo "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$method\",\"arguments\":$params},\"id\":1}" | nc localhost 12700
}

echo -e "\n${YELLOW}Test 1: Graceful shutdown test${NC}"
echo "Creating a process that handles SIGTERM properly..."

# プロセスを作成して起動（JSONをエスケープ）
cat << EOF | ./target/release/ichimi 2>/dev/null | grep -o '"id":"[^"]*"' | cut -d'"' -f4 > /tmp/process_id_1.txt &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"create_process","arguments":{"name":"graceful_test","command":"python3","args":["/tmp/graceful_test.py"],"env":{}}},"id":1}
EOF

PROCESS_ID_1=$(cat /tmp/process_id_1.txt 2>/dev/null || echo "")

if [ -z "$PROCESS_ID_1" ]; then
    echo -e "${RED}Failed to create process${NC}"
    kill $ICHIMI_PID 2>/dev/null
    exit 1
fi

echo "Created process with ID: $PROCESS_ID_1"

# プロセスを起動
echo "Starting the process..."
cat << EOF | ./target/release/ichimi 2>/dev/null &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"start_process","arguments":{"id":"$PROCESS_ID_1"}},"id":2}
EOF

sleep 3

# プロセスの出力を取得
echo "Getting process output..."
cat << EOF | ./target/release/ichimi 2>/dev/null | python3 -m json.tool
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_process_output","arguments":{"id":"$PROCESS_ID_1","max_lines":10}},"id":3}
EOF

# グレースフルシャットダウンをテスト（3秒の猶予期間）
echo -e "\n${YELLOW}Stopping process with 3-second grace period...${NC}"
cat << EOF | ./target/release/ichimi 2>/dev/null &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"stop_process","arguments":{"id":"$PROCESS_ID_1","grace_period_ms":3000}},"id":4}
EOF

sleep 4

# 最終的な出力を確認
echo "Final process output:"
cat << EOF | ./target/release/ichimi 2>/dev/null | python3 -m json.tool
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_process_output","arguments":{"id":"$PROCESS_ID_1","max_lines":20}},"id":5}
EOF

echo -e "\n${YELLOW}Test 2: Stubborn process test (should be force-killed after timeout)${NC}"
echo "Creating a process that ignores SIGTERM..."

# 頑固なプロセスを作成
cat << EOF | ./target/release/ichimi 2>/dev/null | grep -o '"id":"[^"]*"' | cut -d'"' -f4 > /tmp/process_id_2.txt &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"create_process","arguments":{"name":"stubborn_test","command":"python3","args":["/tmp/stubborn_test.py"],"env":{}}},"id":6}
EOF

PROCESS_ID_2=$(cat /tmp/process_id_2.txt 2>/dev/null || echo "")

if [ -z "$PROCESS_ID_2" ]; then
    echo -e "${RED}Failed to create stubborn process${NC}"
    kill $ICHIMI_PID 2>/dev/null
    exit 1
fi

echo "Created stubborn process with ID: $PROCESS_ID_2"

# プロセスを起動
echo "Starting the stubborn process..."
cat << EOF | ./target/release/ichimi 2>/dev/null &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"start_process","arguments":{"id":"$PROCESS_ID_2"}},"id":7}
EOF

sleep 3

# グレースフルシャットダウンを試みる（2秒の猶予期間）
echo -e "\n${YELLOW}Stopping stubborn process with 2-second grace period (should force-kill)...${NC}"
cat << EOF | ./target/release/ichimi 2>/dev/null &
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"stop_process","arguments":{"id":"$PROCESS_ID_2","grace_period_ms":2000}},"id":8}
EOF

sleep 3

# プロセスの状態を確認
echo "Checking process status (should be stopped):"
cat << EOF | ./target/release/ichimi 2>/dev/null | python3 -m json.tool
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_process_status","arguments":{"id":"$PROCESS_ID_2"}},"id":9}
EOF

# クリーンアップ
echo -e "\n${GREEN}Cleaning up...${NC}"
kill $ICHIMI_PID 2>/dev/null || true
rm -f /tmp/graceful_test.py /tmp/stubborn_test.py /tmp/process_id_*.txt /tmp/ichimi_test.log

echo -e "\n${GREEN}=== Test Complete ===${NC}"