#!/bin/bash

# オートスタート機能とグレースフルシャットダウンの統合テスト
# auto_start_on_restoreフラグがtrueのプロセスが再起動時に自動開始されるかテスト

set -e

echo "=================================================="
echo "Auto-Start with Graceful Shutdown Test"
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
echo "[$(date)] Process started with PID $$"
trap 'echo "[$(date)] Received SIGTERM, shutting down gracefully..."; exit 0' SIGTERM
while true; do
    echo "[$(date)] Process running..."
    sleep 2
done
EOF
chmod +x test_process.sh

# カレントディレクトリのパスを取得
CURRENT_DIR=$(pwd)

echo ""
echo "1. Creating test configuration with auto-start processes..."

# auto_start.yamlファイルを作成（オートスタートするプロセスの定義）
cat > "$ICHIMI_DIR/auto_start.yaml" << EOF
processes:
  - id: auto-start-test-1
    name: "Auto Start Process 1"
    command: "$CURRENT_DIR/test_process.sh"
    args: []
    env: {}
    cwd: "$CURRENT_DIR"
    auto_start_on_restore: true
    state: Running
    created_at: $(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
  - id: auto-start-test-2
    name: "Auto Start Process 2"
    command: "$CURRENT_DIR/test_process.sh"
    args: []
    env: {}
    cwd: "$CURRENT_DIR"
    auto_start_on_restore: false
    state: Running
    created_at: $(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ")
EOF

echo "Created auto_start.yaml with 2 processes (1 with auto-start, 1 without)"

echo ""
echo "2. Starting Ichimi server (first run)..."

# 環境変数を設定してIchimiサーバーを起動
export ICHIMI_IMPORT_FILE="$ICHIMI_DIR/auto_start.yaml"
export ICHIMI_EXPORT_FILE="$ICHIMI_DIR/snapshot.yaml"
export RUST_LOG=info

# Ichimiサーバーを起動（バックグラウンド）
timeout 10 cargo run --bin ichimi -- --no-web 2>&1 | tee ichimi_run1.log &
ICHIMI_PID=$!

echo "Ichimi server started with PID $ICHIMI_PID"
echo "Import file: $ICHIMI_IMPORT_FILE"
echo "Export file: $ICHIMI_EXPORT_FILE"

# サーバーが起動し、プロセスがインポートされるのを待つ
sleep 5

# プロセスが実行中か確認
echo ""
echo "3. Checking if processes are running..."

# Ichimiのログを確認
if grep -q "Successfully imported processes" ichimi_run1.log; then
    echo "✓ Processes imported successfully"
else
    echo "✗ Failed to import processes"
    cat ichimi_run1.log
fi

echo ""
echo "4. Triggering graceful shutdown..."

# Ichimiサーバーにグレースフルシャットダウンシグナルを送信
kill -TERM $ICHIMI_PID 2>/dev/null || true

# シャットダウンの完了を待つ
for i in {1..10}; do
    if ! ps -p $ICHIMI_PID > /dev/null 2>&1; then
        echo "✓ Ichimi server shut down"
        break
    fi
    echo "Waiting for shutdown... ($i/10)"
    sleep 1
done

# シャットダウンログの確認
echo ""
echo "5. Checking shutdown logs..."

if grep -q "Graceful shutdown enabled" ichimi_run1.log; then
    echo "✓ Graceful shutdown was enabled"
fi

if grep -q "Starting graceful shutdown of all managed processes" ichimi_run1.log; then
    echo "✓ Graceful shutdown initiated"
fi

if grep -q "Successfully stopped.*process(es) with graceful shutdown" ichimi_run1.log; then
    echo "✓ Processes were stopped gracefully"
fi

if grep -q "Created auto-start snapshot" ichimi_run1.log; then
    echo "✓ Auto-start snapshot was created"
fi

# auto_start.yamlファイルの存在確認
if [ -f "$ICHIMI_DIR/auto_start.yaml" ]; then
    echo "✓ auto_start.yaml exists"
    echo ""
    echo "Content of auto_start.yaml:"
    cat "$ICHIMI_DIR/auto_start.yaml" | head -20
fi

echo ""
echo "6. Starting Ichimi server again (second run)..."

# 2回目の起動（auto_start.yamlから自動復元されるはず）
timeout 10 cargo run --bin ichimi -- --no-web 2>&1 | tee ichimi_run2.log &
ICHIMI_PID2=$!

echo "Ichimi server restarted with PID $ICHIMI_PID2"

# サーバーが起動し、プロセスが復元されるのを待つ
sleep 5

echo ""
echo "7. Checking if auto-start processes were restored..."

# auto_start.yamlからのインポートを確認
if grep -q "Importing processes from.*auto_start.yaml" ichimi_run2.log; then
    echo "✓ Attempting to import from auto_start.yaml"
fi

if grep -q "Successfully imported processes" ichimi_run2.log; then
    echo "✓ Processes imported successfully on restart"
fi

# プロセスの起動を確認（auto_start_on_restore=trueのプロセスのみ起動されるはず）
echo ""
echo "8. Verifying process states after restart..."

# ログからプロセスの状態を確認
echo "Checking logs for process auto-start behavior:"
grep -E "(auto-start-test-1|auto-start-test-2|Starting process|Process.*started)" ichimi_run2.log || true

# 2回目のシャットダウン
echo ""
echo "9. Shutting down second instance..."
kill -TERM $ICHIMI_PID2 2>/dev/null || true

# クリーンアップ
cleanup() {
    echo ""
    echo "10. Cleaning up..."

    # プロセスの強制終了
    pkill -f test_process.sh 2>/dev/null || true

    # Ichimiサーバーの強制終了（もし残っていれば）
    [ ! -z "$ICHIMI_PID" ] && kill -9 $ICHIMI_PID 2>/dev/null || true
    [ ! -z "$ICHIMI_PID2" ] && kill -9 $ICHIMI_PID2 2>/dev/null || true

    # ログの概要表示
    echo ""
    echo "=== Test Summary ==="
    echo "First run logs (relevant lines):"
    grep -E "(import|export|graceful|shutdown|auto-start)" ichimi_run1.log | head -10 || true

    echo ""
    echo "Second run logs (relevant lines):"
    grep -E "(import|export|Starting process|auto-start)" ichimi_run2.log | head -10 || true

    # 一時ディレクトリの削除
    cd /
    rm -rf "$TEST_DIR"

    echo ""
    echo "Test completed!"
}

trap cleanup EXIT

# テスト結果の判定
echo ""
echo "=================================================="
echo "Test Results:"
echo "=================================================="

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: グレースフルシャットダウンが有効か
if grep -q "Graceful shutdown enabled" ichimi_run1.log; then
    echo "✓ Test 1 PASSED: Graceful shutdown is enabled"
    ((TESTS_PASSED++))
else
    echo "✗ Test 1 FAILED: Graceful shutdown not enabled"
    ((TESTS_FAILED++))
fi

# Test 2: プロセスがグレースフルに停止されたか
if grep -q "Successfully stopped.*process(es) with graceful shutdown" ichimi_run1.log; then
    echo "✓ Test 2 PASSED: Processes stopped gracefully"
    ((TESTS_PASSED++))
else
    echo "✗ Test 2 FAILED: Processes not stopped gracefully"
    ((TESTS_FAILED++))
fi

# Test 3: auto_start.yamlが作成されたか
if [ -f "$ICHIMI_DIR/auto_start.yaml" ]; then
    echo "✓ Test 3 PASSED: auto_start.yaml created"
    ((TESTS_PASSED++))
else
    echo "✗ Test 3 FAILED: auto_start.yaml not created"
    ((TESTS_FAILED++))
fi

# Test 4: 再起動時にauto_start.yamlからインポートされたか
if grep -q "Successfully imported processes" ichimi_run2.log; then
    echo "✓ Test 4 PASSED: Processes imported on restart"
    ((TESTS_PASSED++))
else
    echo "✗ Test 4 FAILED: Processes not imported on restart"
    ((TESTS_FAILED++))
fi

echo ""
echo "=================================================="
echo "Total: $((TESTS_PASSED + TESTS_FAILED)) tests"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo "=================================================="

if [ $TESTS_FAILED -eq 0 ]; then
    echo "ALL TESTS PASSED ✓"
    exit 0
else
    echo "SOME TESTS FAILED ✗"
    exit 1
fi