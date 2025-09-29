#!/bin/bash

# Ichimi クイックテスト - tail -fを使った継続動作

set -e

echo "=================================================="
echo "Ichimi Quick Integration Test"
echo "=================================================="

# テスト準備
TEST_DIR=".test_quick"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# テストプロセス作成
cat > "$TEST_DIR/test.sh" << 'EOF'
#!/bin/bash
echo "[$(date +%H:%M:%S)] Test process started (PID $$)"
trap 'echo "[$(date +%H:%M:%S)] SIGTERM received, exiting..."; exit 0' SIGTERM
while true; do
    echo "[$(date +%H:%M:%S)] Running..."
    sleep 2
done
EOF
chmod +x "$TEST_DIR/test.sh"

echo "1. Starting Ichimi with tail -f to keep stdin open..."

# tail -fでstdinを継続的に開いておく
(tail -f /dev/null | RUST_LOG=info ./target/release/ichimi > "$TEST_DIR/ichimi.log" 2>&1) &
ICHIMI_PID=$!

echo "Ichimi started with PID $ICHIMI_PID"
sleep 2

# プロセスが動いているか確認
if ps -p $ICHIMI_PID > /dev/null 2>&1; then
    echo "✓ Ichimi is running continuously"
else
    echo "✗ Ichimi failed to stay running"
    cat "$TEST_DIR/ichimi.log"
    exit 1
fi

# ログを確認
echo ""
echo "2. Checking initialization..."
if grep -q "MCP server is ready" "$TEST_DIR/ichimi.log"; then
    echo "✓ MCP server initialized"
else
    echo "✗ MCP server not ready"
fi

if grep -q "Graceful shutdown enabled" "$TEST_DIR/ichimi.log"; then
    echo "✓ Graceful shutdown is enabled"
else
    echo "✗ Graceful shutdown not enabled"
fi

echo ""
echo "3. Running for 5 seconds to ensure stability..."
sleep 5

if ps -p $ICHIMI_PID > /dev/null 2>&1; then
    echo "✓ Ichimi still running after 5 seconds"
else
    echo "✗ Ichimi crashed"
    tail -20 "$TEST_DIR/ichimi.log"
fi

echo ""
echo "4. Testing graceful shutdown..."
kill -TERM $ICHIMI_PID 2>/dev/null || true

# シャットダウンを待つ
for i in {1..5}; do
    if ! ps -p $ICHIMI_PID > /dev/null 2>&1; then
        echo "✓ Ichimi shut down gracefully after $i second(s)"
        break
    fi
    sleep 1
done

# ログの最後を表示
echo ""
echo "5. Final log output:"
tail -10 "$TEST_DIR/ichimi.log" | grep -E "(shutdown|stopped|Graceful)" || echo "No shutdown logs found"

# クリーンアップ
pkill -f "tail -f /dev/null" 2>/dev/null || true
rm -rf "$TEST_DIR"

echo ""
echo "✓ Quick test completed!"