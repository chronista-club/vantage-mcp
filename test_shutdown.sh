#!/bin/bash

# Ichimiサーバーのグレースフルシャットダウンとオートスタート機能のテスト

set -e

echo "=================================================="
echo "Ichimi Graceful Shutdown & Auto-Start Test"
echo "=================================================="

# プロジェクトルートから実行
PROJECT_DIR=$(pwd)
echo "Project directory: $PROJECT_DIR"

# テスト用ディレクトリの準備
TEST_DIR="$PROJECT_DIR/.test_tmp"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$PROJECT_DIR"

# Ichimiディレクトリ
ICHIMI_DIR="$TEST_DIR/.ichimi"
mkdir -p "$ICHIMI_DIR"

# テストプロセススクリプトの作成
cat > "$TEST_DIR/test_process.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] Process $1 started with PID $$"
trap 'echo "[$(date +%H:%M:%S)] Process $1 received SIGTERM, shutting down gracefully..."; exit 0' SIGTERM
trap 'echo "[$(date +%H:%M:%S)] Process $1 received SIGINT"; exit 0' SIGINT
while true; do
    echo "[$(date +%H:%M:%S)] Process $1 running..."
    sleep 2
done
EOF
chmod +x "$TEST_DIR/test_process.sh"

echo ""
echo "1. Building Ichimi server..."
cargo build --bin ichimi --release

echo ""
echo "2. Creating test configuration..."

# auto_start.yamlファイルを作成
cat > "$ICHIMI_DIR/auto_start.yaml" << EOF
processes:
  - id: auto-process-1
    name: "Auto Start Process"
    command: "$TEST_DIR/test_process.sh"
    args: ["auto-1"]
    env: {}
    cwd: "$TEST_DIR"
    auto_start_on_restore: true
    state: Running
    created_at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
  - id: manual-process-1
    name: "Manual Process"
    command: "$TEST_DIR/test_process.sh"
    args: ["manual-1"]
    env: {}
    cwd: "$TEST_DIR"
    auto_start_on_restore: false
    state: Running
    created_at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF

echo "Created test configuration with 2 processes:"
echo "  - auto-process-1: auto_start_on_restore = true"
echo "  - manual-process-1: auto_start_on_restore = false"

echo ""
echo "3. Starting Ichimi server (first run)..."

# 環境変数を設定
export ICHIMI_IMPORT_FILE="$ICHIMI_DIR/auto_start.yaml"
export ICHIMI_EXPORT_FILE="$ICHIMI_DIR/snapshot.yaml"
export RUST_LOG=info

# Ichimiサーバーを起動
./target/release/ichimi --no-web > "$TEST_DIR/ichimi_run1.log" 2>&1 &
ICHIMI_PID=$!

echo "Ichimi server started with PID $ICHIMI_PID"
echo "Waiting for initialization..."
sleep 3

# プロセスが起動されたか確認
echo ""
echo "4. Checking initial import..."
if grep -q "Successfully imported processes" "$TEST_DIR/ichimi_run1.log"; then
    echo "✓ Processes imported successfully"
else
    echo "✗ Failed to import processes"
fi

echo ""
echo "5. Triggering graceful shutdown (Ctrl+C)..."
kill -TERM $ICHIMI_PID

# シャットダウンを待つ
SHUTDOWN_COMPLETE=false
for i in {1..10}; do
    if ! ps -p $ICHIMI_PID > /dev/null 2>&1; then
        echo "✓ Ichimi server shut down after $i second(s)"
        SHUTDOWN_COMPLETE=true
        break
    fi
    sleep 1
done

if [ "$SHUTDOWN_COMPLETE" = false ]; then
    echo "✗ Ichimi did not shut down gracefully, forcing..."
    kill -9 $ICHIMI_PID 2>/dev/null || true
fi

echo ""
echo "6. Analyzing first run logs..."
echo "Key events from first run:"
grep -E "(Graceful|graceful|shutdown|Shutdown|stopped|auto-start|snapshot)" "$TEST_DIR/ichimi_run1.log" | head -20 || echo "No relevant logs found"

# auto_start.yamlが作成されたか確認
echo ""
echo "7. Checking for auto-start snapshot..."
if [ -f "$ICHIMI_DIR/auto_start.yaml" ]; then
    echo "✓ auto_start.yaml exists"
    echo "First 30 lines:"
    head -30 "$ICHIMI_DIR/auto_start.yaml"
else
    echo "✗ auto_start.yaml not found"
fi

echo ""
echo "8. Starting Ichimi server (second run - should auto-start processes)..."

# 2回目の起動
./target/release/ichimi --no-web > "$TEST_DIR/ichimi_run2.log" 2>&1 &
ICHIMI_PID2=$!

echo "Ichimi server restarted with PID $ICHIMI_PID2"
echo "Waiting for auto-start to complete..."
sleep 5

echo ""
echo "9. Checking second run for auto-start behavior..."
echo "Key events from second run:"
grep -E "(import|Import|auto-start|Starting process|Process.*started)" "$TEST_DIR/ichimi_run2.log" | head -20 || echo "No auto-start logs found"

# 2回目もシャットダウン
echo ""
echo "10. Shutting down second instance..."
kill -TERM $ICHIMI_PID2 2>/dev/null || true
sleep 2
kill -9 $ICHIMI_PID2 2>/dev/null || true

echo ""
echo "=================================================="
echo "Test Results Summary"
echo "=================================================="

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: グレースフルシャットダウンが有効か
if grep -q "Graceful shutdown enabled" "$TEST_DIR/ichimi_run1.log"; then
    echo "✓ Test 1 PASSED: Graceful shutdown is enabled by default"
    ((TESTS_PASSED++))
else
    echo "✗ Test 1 FAILED: Graceful shutdown not enabled"
    ((TESTS_FAILED++))
fi

# Test 2: プロセスがグレースフルに停止されたか
if grep -q "graceful" "$TEST_DIR/ichimi_run1.log"; then
    echo "✓ Test 2 PASSED: Graceful shutdown was performed"
    ((TESTS_PASSED++))
else
    echo "✗ Test 2 FAILED: No graceful shutdown detected"
    ((TESTS_FAILED++))
fi

# Test 3: auto-start snapshotが作成されたか
if grep -q "auto-start snapshot" "$TEST_DIR/ichimi_run1.log"; then
    echo "✓ Test 3 PASSED: Auto-start snapshot was created"
    ((TESTS_PASSED++))
else
    echo "✗ Test 3 FAILED: Auto-start snapshot not created"
    ((TESTS_FAILED++))
fi

# Test 4: 2回目起動時にプロセスがインポートされたか
if grep -q "imported" "$TEST_DIR/ichimi_run2.log"; then
    echo "✓ Test 4 PASSED: Processes were imported on restart"
    ((TESTS_PASSED++))
else
    echo "✗ Test 4 FAILED: No import on restart"
    ((TESTS_FAILED++))
fi

echo ""
echo "--------------------------------------------------"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo "--------------------------------------------------"

# クリーンアップ
echo ""
echo "11. Cleaning up..."
pkill -f test_process.sh 2>/dev/null || true
rm -rf "$TEST_DIR"

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo "✓✓✓ ALL TESTS PASSED ✓✓✓"
    exit 0
else
    echo ""
    echo "✗✗✗ SOME TESTS FAILED ✗✗✗"
    exit 1
fi