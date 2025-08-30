#!/bin/bash

# MCP Protocol Test for Ichimi Server with Enhanced Logging

# Build the server first
echo "Building Ichimi Server..."
cargo build --release

# Create logs directory if it doesn't exist
mkdir -p ~/.ichimi/logs

# Set environment to simulate MCP
export MCP_SERVER_NAME="ichimi-test"
export RUST_LOG="debug"

# Function to send JSON-RPC message
send_message() {
    local message=$1
    echo "$message" | ./target/release/ichimi --no-web 2>&1 | grep -v "^\[ICHIMI\]"
}

echo ""
echo "Testing Ichimi MCP Server..."
echo "==============================="
echo "Log file will be created in: ~/.ichimi/logs/"
echo "==============================="

# 1. Initialize
echo "1. Sending initialize..."
send_message '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'
sleep 1

# 2. Echo test
echo -e "\n2. Testing echo tool..."
send_message '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"echo","arguments":{"message":"Hello from MCP test"}},"id":2}'
sleep 1

# 3. Ping test
echo -e "\n3. Testing ping tool..."
send_message '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"ping","arguments":{}},"id":3}'
sleep 1

# 4. Get status
echo -e "\n4. Testing get_status tool..."
send_message '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"get_status","arguments":{}},"id":4}'
sleep 1

# 5. List processes
echo -e "\n5. Testing list_processes tool..."
send_message '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_processes","arguments":{}},"id":5}'

echo -e "\n==============================="
echo "Test completed!"
echo ""
echo "To view the log file:"
echo "-------------------------------"
LATEST_LOG=$(ls -t ~/.ichimi/logs/ichimi-mcp-*.log 2>/dev/null | head -1)
if [ -n "$LATEST_LOG" ]; then
    echo "Latest log: $LATEST_LOG"
    echo ""
    echo "View with: tail -f $LATEST_LOG"
    echo ""
    echo "First 20 lines of log:"
    echo "-------------------------------"
    head -20 "$LATEST_LOG"
else
    echo "No log file found. Check ~/.ichimi/logs/"
fi