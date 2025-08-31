#!/bin/bash

echo "=== Testing ichimi shutdown behavior (detailed) ==="

# Cleanup function
cleanup() {
    echo "Cleaning up test processes..."
    pkill -f "sleep 9999" 2>/dev/null
    pkill -f "test_long_runner" 2>/dev/null
    rm -f /tmp/test_process_*.txt /tmp/test_long_runner.sh
    pkill -f "ichimi.*--web" 2>/dev/null
}

# Set trap for cleanup
trap cleanup EXIT

# Build if needed
echo "Building ichimi..."
cargo build --release

echo ""
echo "Starting ichimi with web dashboard and debug logging..."
RUST_LOG=debug ./target/release/ichimi --web-only --no-open --web-port 12710 &
ICHIMI_PID=$!

# Wait for server to start
sleep 3

echo ""
echo "Server PID: $ICHIMI_PID"
echo "Checking if server is running..."
ps aux | grep $ICHIMI_PID | grep -v grep

echo ""
echo "Creating test script that will keep running..."
cat > /tmp/test_long_runner.sh << 'EOF'
#!/bin/bash
echo "Test process started at $(date)"
while true; do
    echo "Still running at $(date)"
    sleep 5
done
EOF
chmod +x /tmp/test_long_runner.sh

echo ""
echo "Creating and starting test processes via API..."

# Create first process
echo "Creating test-process-1 (sleep 9999)..."
curl -X POST http://localhost:12710/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-process-1",
    "command": "sleep",
    "args": ["9999"],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }' 
echo ""

sleep 1

# Create second process
echo "Creating test-process-2 (long runner script)..."
curl -X POST http://localhost:12710/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-process-2", 
    "command": "/tmp/test_long_runner.sh",
    "args": [],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }'
echo ""

sleep 2

echo ""
echo "Listing all processes via API..."
curl -s http://localhost:12710/api/processes | python3 -m json.tool || echo "Failed to get process list"

echo ""
echo "Checking running processes in OS..."
echo "Sleep processes:"
ps aux | grep "sleep 9999" | grep -v grep
echo "Script processes:"
ps aux | grep "test_long_runner" | grep -v grep

echo ""
echo "Sending shutdown signal to ichimi (Ctrl+C)..."
kill -SIGINT $ICHIMI_PID

# Wait for ichimi to shutdown
echo "Waiting for shutdown to complete..."
sleep 5

echo ""
echo "Checking if ichimi process is gone..."
if ps aux | grep $ICHIMI_PID | grep -v grep > /dev/null; then
    echo "❌ ichimi is still running!"
else
    echo "✅ ichimi has stopped"
fi

echo ""
echo "Checking if managed processes are still running..."
SLEEP_REMAINING=$(ps aux | grep "sleep 9999" | grep -v grep | wc -l)
SCRIPT_REMAINING=$(ps aux | grep "test_long_runner" | grep -v grep | wc -l)
TOTAL_REMAINING=$((SLEEP_REMAINING + SCRIPT_REMAINING))

if [ $TOTAL_REMAINING -eq 0 ]; then
    echo "✅ SUCCESS: All managed processes were properly stopped!"
else
    echo "❌ FAILURE: $TOTAL_REMAINING process(es) still running after ichimi shutdown:"
    echo "  Sleep processes: $SLEEP_REMAINING"
    echo "  Script processes: $SCRIPT_REMAINING"
    echo ""
    echo "Remaining processes:"
    ps aux | grep -E "(sleep 9999|test_long_runner)" | grep -v grep
fi

echo ""
echo "Checking exported data..."
if [ -f ~/.ichimi/data/processes.surql ]; then
    echo "Export file exists. Content:"
    cat ~/.ichimi/data/processes.surql
else
    echo "No export file found"
fi

echo ""
echo "Test complete."