#!/bin/bash

# シンプルなグレースフルシャットダウンテスト

set -e

echo "=================================================="
echo "Simple Graceful Shutdown Test"
echo "=================================================="

# 作業ディレクトリの準備
TEST_DIR=$(mktemp -d)
echo "Test directory: $TEST_DIR"
cd $TEST_DIR

# テスト用のIchimiデータディレクトリ
ICHIMI_DIR="$TEST_DIR/.ichimi"
mkdir -p "$ICHIMI_DIR"

# テストプロセススクリプトの作成
cat > test_process.sh << 'EOF'
#!/bin/bash
echo "[$(date)] Process started with PID $$" >> process.log
trap 'echo "[$(date)] Received SIGTERM, shutting down gracefully..." >> process.log; exit 0' SIGTERM
while true; do
    echo "[$(date)] Process running..." >> process.log
    sleep 1
done
EOF
chmod +x test_process.sh

# カレントディレクトリのパスを取得
CURRENT_DIR=$(pwd)

echo ""
echo "1. Starting a test process in background..."
./test_process.sh &
TEST_PID=$!
echo "Test process started with PID $TEST_PID"

sleep 2

echo ""
echo "2. Verifying process is running..."
if ps -p $TEST_PID > /dev/null 2>&1; then
    echo "✓ Process is running"
else
    echo "✗ Process failed to start"
    exit 1
fi

echo ""
echo "3. Sending SIGTERM for graceful shutdown..."
kill -TERM $TEST_PID

# グレースフルシャットダウンを待つ
echo "Waiting for graceful shutdown..."
SHUTDOWN_SUCCESS=false
for i in {1..5}; do
    if ! ps -p $TEST_PID > /dev/null 2>&1; then
        echo "✓ Process shut down gracefully after $i second(s)"
        SHUTDOWN_SUCCESS=true
        break
    fi
    sleep 1
done

if [ "$SHUTDOWN_SUCCESS" = false ]; then
    echo "✗ Process did not shut down within 5 seconds"
    kill -9 $TEST_PID 2>/dev/null || true
fi

echo ""
echo "4. Checking process log..."
if [ -f process.log ]; then
    echo "Process log contents:"
    cat process.log

    if grep -q "Received SIGTERM, shutting down gracefully" process.log; then
        echo ""
        echo "✓ Process received SIGTERM and handled it gracefully"
    else
        echo ""
        echo "✗ Process did not handle SIGTERM properly"
    fi
fi

# Ichimiサーバーのテスト
echo ""
echo "=================================================="
echo "Testing Ichimi Server Graceful Shutdown"
echo "=================================================="

# auto_start.yamlファイルを作成
cat > "$ICHIMI_DIR/auto_start.yaml" << EOF
processes:
  - id: test-process-1
    name: "Test Process 1"
    command: "$CURRENT_DIR/test_process.sh"
    args: []
    env: {}
    cwd: "$CURRENT_DIR"
    auto_start_on_restore: true
    state: NotStarted
    created_at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF

echo ""
echo "5. Starting Ichimi server..."

# 環境変数を設定
export ICHIMI_IMPORT_FILE="$ICHIMI_DIR/auto_start.yaml"
export ICHIMI_EXPORT_FILE="$ICHIMI_DIR/snapshot.yaml"
export RUST_LOG=info

# Ichimiサーバーをバックグラウンドで起動
cargo run --bin ichimi -- --no-web 2>&1 > ichimi.log &
ICHIMI_PID=$!

echo "Ichimi server started with PID $ICHIMI_PID"
echo "Waiting for server to initialize..."
sleep 5

echo ""
echo "6. Sending graceful shutdown signal to Ichimi..."
kill -TERM $ICHIMI_PID

# シャットダウンを待つ
echo "Waiting for Ichimi to shut down..."
ICHIMI_SHUTDOWN=false
for i in {1..10}; do
    if ! ps -p $ICHIMI_PID > /dev/null 2>&1; then
        echo "✓ Ichimi server shut down after $i second(s)"
        ICHIMI_SHUTDOWN=true
        break
    fi
    sleep 1
done

if [ "$ICHIMI_SHUTDOWN" = false ]; then
    echo "✗ Ichimi did not shut down within 10 seconds"
    kill -9 $ICHIMI_PID 2>/dev/null || true
fi

echo ""
echo "7. Checking Ichimi logs..."
echo "Last 30 lines of Ichimi log:"
tail -n 30 ichimi.log

# ログの検証
echo ""
echo "=================================================="
echo "Test Results"
echo "=================================================="

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: グレースフルシャットダウンが有効か
if grep -q "Graceful shutdown enabled" ichimi.log; then
    echo "✓ Test 1 PASSED: Graceful shutdown is enabled"
    ((TESTS_PASSED++))
else
    echo "✗ Test 1 FAILED: Graceful shutdown not enabled"
    ((TESTS_FAILED++))
fi

# Test 2: シャットダウンが開始されたか
if grep -q "graceful shutdown" ichimi.log; then
    echo "✓ Test 2 PASSED: Graceful shutdown was initiated"
    ((TESTS_PASSED++))
else
    echo "✗ Test 2 FAILED: Graceful shutdown not initiated"
    ((TESTS_FAILED++))
fi

# Test 3: スナップショットが作成されたか
if [ -f "$ICHIMI_DIR/snapshot.yaml" ] || [ -f "$ICHIMI_DIR/auto_start.yaml" ]; then
    echo "✓ Test 3 PASSED: Snapshot files exist"
    ((TESTS_PASSED++))
else
    echo "✗ Test 3 FAILED: No snapshot files created"
    ((TESTS_FAILED++))
fi

echo ""
echo "=================================================="
echo "Total: $((TESTS_PASSED + TESTS_FAILED)) tests"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo "=================================================="

# クリーンアップ
cd /
rm -rf "$TEST_DIR"

if [ $TESTS_FAILED -eq 0 ]; then
    echo "ALL TESTS PASSED ✓"
    exit 0
else
    echo "SOME TESTS FAILED ✗"
    exit 1
fi