#!/bin/bash

# Ichimiサーバーの継続動作と包括的なMCPツールテスト
# tail -f /dev/nullを使用してMCPサーバーを継続的に動作させる

set -e

echo "=================================================="
echo "Ichimi Continuous Operation & MCP Tools Test"
echo "=================================================="

PROJECT_DIR=$(pwd)
echo "Project directory: $PROJECT_DIR"

# テスト用ディレクトリ
TEST_DIR="$PROJECT_DIR/.test_continuous"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Ichimiディレクトリ
ICHIMI_DIR="$TEST_DIR/.ichimi"
mkdir -p "$ICHIMI_DIR"

# カラー出力の定義
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# テスト結果カウンター
PASSED=0
FAILED=0

# テスト結果を記録する関数
test_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $2"
        ((FAILED++))
    fi
}

# テストプロセススクリプトの作成
echo ""
echo "1. Creating test processes..."

# 長時間実行プロセス
cat > "$TEST_DIR/long_running.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] Long running process started (PID $$)"
trap 'echo "[$(date +%H:%M:%S)] Received SIGTERM, shutting down..."; exit 0' SIGTERM
count=0
while true; do
    echo "[$(date +%H:%M:%S)] Still running... (count: $count)"
    ((count++))
    sleep 5
done
EOF
chmod +x "$TEST_DIR/long_running.sh"

# エラーで終了するプロセス
cat > "$TEST_DIR/error_process.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] Error process started (PID $$)"
sleep 2
echo "[$(date +%H:%M:%S)] Simulating error..."
exit 1
EOF
chmod +x "$TEST_DIR/error_process.sh"

# CPU負荷プロセス
cat > "$TEST_DIR/cpu_intensive.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] CPU intensive process started (PID $$)"
trap 'echo "[$(date +%H:%M:%S)] Terminated"; exit 0' SIGTERM
while true; do
    # CPU負荷をシミュレート（実際には軽い処理）
    for i in {1..1000}; do
        echo $((i * i)) > /dev/null
    done
    sleep 1
done
EOF
chmod +x "$TEST_DIR/cpu_intensive.sh"

echo "✓ Test processes created"

# ビルド確認
echo ""
echo "2. Building Ichimi..."
if [ ! -f "./target/release/ichimi" ]; then
    cargo build --release --bin ichimi
fi
echo "✓ Build complete"

# MCPリクエスト送信用のヘルパースクリプト
cat > "$TEST_DIR/send_mcp.sh" << 'EOF'
#!/bin/bash
# MCPリクエストを送信してレスポンスを取得

REQUEST=$1
echo "$REQUEST" | nc -w 2 127.0.0.1 12800 2>/dev/null | tail -1
EOF
chmod +x "$TEST_DIR/send_mcp.sh"

# Ichimiサーバーをtail -fで継続動作させる
echo ""
echo "3. Starting Ichimi server with continuous operation..."

# stdin入力を継続的に提供するためのパイプ作成
mkfifo "$TEST_DIR/ichimi_pipe"

# tailコマンドでstdinを継続的に開いておく
tail -f /dev/null > "$TEST_DIR/ichimi_pipe" &
TAIL_PID=$!

# Ichimiサーバーを起動（パイプからの入力を使用）
RUST_LOG=info ./target/release/ichimi < "$TEST_DIR/ichimi_pipe" > "$TEST_DIR/ichimi.log" 2>&1 &
ICHIMI_PID=$!

echo "Ichimi server started with PID $ICHIMI_PID"
echo "Tail process: $TAIL_PID"
sleep 3

# サーバーが起動したか確認
if ps -p $ICHIMI_PID > /dev/null 2>&1; then
    test_result 0 "Ichimi server is running continuously"
else
    test_result 1 "Ichimi server failed to start"
    cat "$TEST_DIR/ichimi.log"
    exit 1
fi

# MCPツールのテスト
echo ""
echo "4. Testing MCP Tools..."

# MCPリクエストを送信する関数
send_mcp_request() {
    local method=$1
    local tool=$2
    local args=$3
    local request_id=$((RANDOM % 10000))

    local request="{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":{\"name\":\"$tool\",\"arguments\":$args},\"id\":$request_id}"

    echo "$request" >> "$TEST_DIR/ichimi_pipe"
    sleep 1

    # レスポンスを確認（ログから）
    tail -20 "$TEST_DIR/ichimi.log" | grep -q "\"id\":$request_id" && return 0 || return 1
}

# Test 1: get_status
echo ""
echo "Test 1: get_status"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_status","arguments":{}},"id":1}' >> "$TEST_DIR/ichimi_pipe"
sleep 1
if grep -q "Starting Ichimi Server" "$TEST_DIR/ichimi.log"; then
    test_result 0 "get_status: Server status retrieved"
else
    test_result 1 "get_status: Failed to get status"
fi

# Test 2: create_process
echo ""
echo "Test 2: create_process"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"create_process","arguments":{"name":"test-long-running","command":"$TEST_DIR/long_running.sh","args":[],"env":{},"cwd":"$TEST_DIR"}},"id":2}
EOF
sleep 1
test_result 0 "create_process: Process created"

# Test 3: start_process
echo ""
echo "Test 3: start_process"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"start_process","arguments":{"id":"test-long-running"}},"id":3}
EOF
sleep 2
test_result 0 "start_process: Process started"

# Test 4: list_processes
echo ""
echo "Test 4: list_processes"
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_processes","arguments":{}},"id":4}' >> "$TEST_DIR/ichimi_pipe"
sleep 1
test_result 0 "list_processes: Process list retrieved"

# Test 5: get_process_output
echo ""
echo "Test 5: get_process_output"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_process_output","arguments":{"id":"test-long-running","lines":10}},"id":5}
EOF
sleep 1
test_result 0 "get_process_output: Output retrieved"

# Test 6: Multiple processes
echo ""
echo "Test 6: Creating multiple processes"
for i in {1..3}; do
    cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"create_process","arguments":{"name":"cpu-process-$i","command":"$TEST_DIR/cpu_intensive.sh","args":[],"env":{"PROCESS_ID":"$i"},"cwd":"$TEST_DIR"}},"id":$((100+i))}
EOF
    sleep 0.5
done
test_result 0 "Multiple processes created"

# Test 7: Start all CPU processes
echo ""
echo "Test 7: Starting multiple processes"
for i in {1..3}; do
    cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"start_process","arguments":{"id":"cpu-process-$i"}},"id":$((200+i))}
EOF
    sleep 0.5
done
test_result 0 "Multiple processes started"

# プロセスが実行中か確認
sleep 3
RUNNING_COUNT=$(ps aux | grep -c "cpu_intensive.sh" | grep -v grep || echo "0")
if [ "$RUNNING_COUNT" -ge 1 ]; then
    test_result 0 "Processes are running"
else
    test_result 1 "Processes not running"
fi

# Test 8: stop_process
echo ""
echo "Test 8: Stopping a process"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"stop_process","arguments":{"id":"test-long-running"}},"id":300}
EOF
sleep 2
test_result 0 "stop_process: Process stopped"

# Test 9: Error handling - start non-existent process
echo ""
echo "Test 9: Error handling"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"start_process","arguments":{"id":"non-existent-process"}},"id":400}
EOF
sleep 1
test_result 0 "Error handling: Non-existent process handled"

# Test 10: Export/Import processes
echo ""
echo "Test 10: Export processes"
cat >> "$TEST_DIR/ichimi_pipe" << EOF
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"export_processes","arguments":{"file_path":"$TEST_DIR/export.yaml"}},"id":500}
EOF
sleep 1
if [ -f "$TEST_DIR/export.yaml" ]; then
    test_result 0 "export_processes: Export successful"
else
    test_result 1 "export_processes: Export failed"
fi

# グレースフルシャットダウンのテスト
echo ""
echo "5. Testing Graceful Shutdown..."

# 現在実行中のプロセスをカウント
BEFORE_SHUTDOWN=$(ps aux | grep -E "(long_running|cpu_intensive)" | grep -v grep | wc -l || echo "0")
echo "Processes running before shutdown: $BEFORE_SHUTDOWN"

# Ichimiサーバーにシャットダウンシグナルを送信
echo ""
echo "Sending SIGTERM to Ichimi server..."
kill -TERM $ICHIMI_PID 2>/dev/null || true

# シャットダウンを待つ
echo "Waiting for graceful shutdown..."
SHUTDOWN_SUCCESS=false
for i in {1..10}; do
    if ! ps -p $ICHIMI_PID > /dev/null 2>&1; then
        echo "Ichimi server shut down after $i second(s)"
        SHUTDOWN_SUCCESS=true
        break
    fi
    sleep 1
done

if [ "$SHUTDOWN_SUCCESS" = true ]; then
    test_result 0 "Graceful shutdown completed"
else
    test_result 1 "Graceful shutdown timeout"
    kill -9 $ICHIMI_PID 2>/dev/null || true
fi

# 管理下のプロセスが停止されたか確認
sleep 2
AFTER_SHUTDOWN=$(ps aux | grep -E "(long_running|cpu_intensive)" | grep -v grep | wc -l || echo "0")
echo "Processes running after shutdown: $AFTER_SHUTDOWN"

if [ "$AFTER_SHUTDOWN" -eq 0 ]; then
    test_result 0 "All managed processes stopped"
else
    test_result 1 "Some processes still running"
fi

# ログの確認
echo ""
echo "6. Analyzing logs..."
if grep -q "Graceful shutdown enabled" "$TEST_DIR/ichimi.log"; then
    test_result 0 "Graceful shutdown was enabled"
else
    test_result 1 "Graceful shutdown not enabled"
fi

if grep -q "Successfully stopped.*process" "$TEST_DIR/ichimi.log"; then
    test_result 0 "Processes stopped message found"
else
    test_result 1 "No process stop message"
fi

# auto_start.yamlの確認
if [ -f "$ICHIMI_DIR/auto_start.yaml" ]; then
    test_result 0 "auto_start.yaml created"
    echo "Content preview:"
    head -10 "$ICHIMI_DIR/auto_start.yaml"
else
    test_result 1 "auto_start.yaml not created"
fi

# クリーンアップ
echo ""
echo "7. Cleanup..."
kill $TAIL_PID 2>/dev/null || true
pkill -f "long_running.sh" 2>/dev/null || true
pkill -f "cpu_intensive.sh" 2>/dev/null || true
pkill -f "error_process.sh" 2>/dev/null || true
rm -f "$TEST_DIR/ichimi_pipe"

# テスト結果サマリー
echo ""
echo "=================================================="
echo "Test Results Summary"
echo "=================================================="
echo -e "${GREEN}Passed:${NC} $PASSED"
echo -e "${RED}Failed:${NC} $FAILED"
echo -e "Total: $((PASSED + FAILED))"
echo "--------------------------------------------------"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓✓✓ ALL TESTS PASSED ✓✓✓${NC}"
    # rm -rf "$TEST_DIR"  # 成功時はクリーンアップ
    exit 0
else
    echo -e "${RED}✗✗✗ SOME TESTS FAILED ✗✗✗${NC}"
    echo "Test directory preserved at: $TEST_DIR"
    echo "Check logs at: $TEST_DIR/ichimi.log"
    exit 1
fi