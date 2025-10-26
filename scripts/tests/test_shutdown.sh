#!/bin/bash

echo "=== Testing vantage shutdown behavior ==="

# Cleanup function
cleanup() {
    echo "Cleaning up test processes..."
    pkill -f "sleep 9999" 2>/dev/null
    rm -f /tmp/test_process_*.txt
}

# Set trap for cleanup
trap cleanup EXIT

# Build if needed
echo "Building vantage..."
cargo build --release

echo ""
echo "Starting vantage with web dashboard..."
RUST_LOG=info ./target/release/vantagemcp --no-open &
VANTAGE_PID=$!

# Wait for server to start
sleep 2

echo ""
echo "Creating test processes via MCP..."

# Create test script that will keep running
cat > /tmp/test_long_runner.sh << 'EOF'
#!/bin/bash
echo "Test process started at $(date)"
while true; do
    echo "Still running at $(date)"
    sleep 5
done
EOF
chmod +x /tmp/test_long_runner.sh

# Use the vantage CLI to create and start processes
echo "Creating and starting test processes..."
curl -X POST http://localhost:12700/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-process-1",
    "command": "sleep",
    "args": ["9999"],
    "auto_start_on_create": true
  }' 2>/dev/null

sleep 1

curl -X POST http://localhost:12700/api/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-process-2",
    "command": "/tmp/test_long_runner.sh",
    "args": [],
    "auto_start_on_create": true
  }' 2>/dev/null

sleep 2

echo ""
echo "Checking running processes..."
ps aux | grep -E "(sleep 9999|test_long_runner)" | grep -v grep

echo ""
echo "Sending shutdown signal to vantage (Ctrl+C)..."
kill -SIGINT $VANTAGE_PID

# Wait for vantage to shutdown
sleep 3

echo ""
echo "Checking if processes are still running after vantage shutdown..."
REMAINING=$(ps aux | grep -E "(sleep 9999|test_long_runner)" | grep -v grep | wc -l)

if [ $REMAINING -eq 0 ]; then
    echo "✅ SUCCESS: All managed processes were properly stopped!"
else
    echo "❌ FAILURE: $REMAINING process(es) still running after vantage shutdown:"
    ps aux | grep -E "(sleep 9999|test_long_runner)" | grep -v grep
fi

echo ""
echo "Test complete."