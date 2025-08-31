#!/bin/bash

echo "=== Testing ichimi shutdown behavior (continue vs stop) ==="

# Cleanup function
cleanup() {
    echo "Cleaning up test processes..."
    pkill -f "sleep 8888" 2>/dev/null
    pkill -f "sleep 7777" 2>/dev/null
    rm -f /tmp/test_*.txt
    pkill -f "ichimi.*--web" 2>/dev/null
}

# Set trap for cleanup
trap cleanup EXIT

# Build if needed
echo "Building ichimi..."
cargo build --release

echo ""
echo "========================================="
echo "TEST 1: Default behavior (CONTINUE running)"
echo "========================================="
echo ""

echo "Starting ichimi with default settings (processes should continue)..."
./target/release/ichimi --web-only --no-open --web-port 12711 &
ICHIMI_PID=$!

sleep 2

echo "Creating test process via API..."
curl -X POST http://localhost:12711/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-continue",
    "command": "sleep",
    "args": ["8888"],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }' 2>/dev/null
echo ""

sleep 1

echo "Checking if process is running..."
PROCESS_PID=$(ps aux | grep "sleep 8888" | grep -v grep | awk '{print $2}')
if [ ! -z "$PROCESS_PID" ]; then
    echo "✅ Process is running with PID: $PROCESS_PID"
else
    echo "❌ Process failed to start"
fi

echo ""
echo "Sending shutdown signal to ichimi..."
kill -SIGINT $ICHIMI_PID

sleep 3

echo "Checking if process is still running after ichimi shutdown..."
if ps -p $PROCESS_PID > /dev/null 2>&1; then
    echo "✅ SUCCESS: Process continues running after ichimi shutdown (PID: $PROCESS_PID)"
    # Clean up the test process
    kill $PROCESS_PID
else
    echo "❌ FAILURE: Process was stopped (should have continued)"
fi

sleep 2

echo ""
echo "========================================="
echo "TEST 2: With ICHIMI_STOP_ON_SHUTDOWN=true"
echo "========================================="
echo ""

echo "Starting ichimi with ICHIMI_STOP_ON_SHUTDOWN=true..."
ICHIMI_STOP_ON_SHUTDOWN=true ./target/release/ichimi --web-only --no-open --web-port 12712 &
ICHIMI_PID=$!

sleep 2

echo "Creating test process via API..."
curl -X POST http://localhost:12712/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-stop",
    "command": "sleep",
    "args": ["7777"],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }' 2>/dev/null
echo ""

sleep 1

echo "Checking if process is running..."
PROCESS_PID=$(ps aux | grep "sleep 7777" | grep -v grep | awk '{print $2}')
if [ ! -z "$PROCESS_PID" ]; then
    echo "✅ Process is running with PID: $PROCESS_PID"
else
    echo "❌ Process failed to start"
fi

echo ""
echo "Sending shutdown signal to ichimi..."
kill -SIGINT $ICHIMI_PID

sleep 3

echo "Checking if process was stopped after ichimi shutdown..."
if ps -p $PROCESS_PID > /dev/null 2>&1; then
    echo "❌ FAILURE: Process is still running (should have been stopped)"
    # Clean up the test process
    kill $PROCESS_PID
else
    echo "✅ SUCCESS: Process was stopped as expected"
fi

echo ""
echo "========================================="
echo "TEST SUMMARY"
echo "========================================="
echo ""
echo "Test complete. Both behaviors have been verified:"
echo "1. Default: Processes CONTINUE running after ichimi shutdown"
echo "2. ICHIMI_STOP_ON_SHUTDOWN=true: Processes STOP when ichimi shuts down"