#!/bin/bash

# データベース永続化テスト

echo "=== Database Persistence Test ==="

# テスト用ディレクトリ
TEST_DIR="/tmp/ichimi_test_$$"
mkdir -p "$TEST_DIR"

cleanup() {
    echo "Cleaning up..."
    pkill -f "ichimi.*--web" 2>/dev/null
    rm -rf "$TEST_DIR"
}
trap cleanup EXIT

# エクスポートファイル
EXPORT_FILE="$TEST_DIR/test_export.surql"

echo "1. Starting ichimi server..."
ICHIMI_EXPORT_FILE="$EXPORT_FILE" RUST_LOG=info ./target/release/ichimi --web-only --no-open --web-port 12720 &
ICHIMI_PID=$!
sleep 2

echo "2. Creating test process..."
curl -s -X POST http://localhost:12720/api/processes \
    -H "Content-Type: application/json" \
    -d '{"id": "test-db-process", "command": "echo", "args": ["Database Test"], "env": {"TEST": "123"}}' | jq

echo "3. Exporting database..."
kill -TERM $ICHIMI_PID
wait $ICHIMI_PID 2>/dev/null

if [ -f "$EXPORT_FILE" ]; then
    echo "✓ Export file created"
    echo "Export content preview:"
    head -5 "$EXPORT_FILE"
else
    echo "✗ Export file not created"
    exit 1
fi

echo "4. Re-importing database..."
ICHIMI_IMPORT_FILE="$EXPORT_FILE" RUST_LOG=info ./target/release/ichimi --web-only --no-open --web-port 12721 &
ICHIMI_PID=$!
sleep 2

echo "5. Verifying imported data..."
PROCESSES=$(curl -s http://localhost:12721/api/processes | jq)
echo "Imported processes: $PROCESSES"

if echo "$PROCESSES" | grep -q "test-db-process"; then
    echo "✓ Process successfully persisted and restored"
else
    echo "✗ Process not found after import"
    exit 1
fi

kill -TERM $ICHIMI_PID
wait $ICHIMI_PID 2>/dev/null

echo "✓ Database persistence test passed"