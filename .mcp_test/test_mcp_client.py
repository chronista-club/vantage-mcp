#!/usr/bin/env python3
import json
import subprocess
import sys
import time

def send_mcp_request(method, params=None):
    """Send MCP request to ichimi server"""
    request = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params or {},
        "id": 1
    }

    # ichimiサーバーはstdin/stdoutを使用
    proc = subprocess.Popen(
        ["./target/release/ichimi", "--no-web"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # リクエストを送信
    proc.stdin.write(json.dumps(request) + "\n")
    proc.stdin.flush()

    # レスポンスを読み取り（タイムアウト付き）
    response = proc.stdout.readline()
    proc.terminate()

    if response:
        return json.loads(response)
    return None

# Test 1: Create process
print("1. Creating test process...")
response = send_mcp_request("tools/call", {
    "name": "create_process",
    "arguments": {
        "name": "test-process",
        "command": sys.argv[1] + "/test_process.sh",
        "args": [],
        "env": {},
        "cwd": sys.argv[1]
    }
})
print(f"Response: {response}")

# Test 2: List processes
print("\n2. Listing processes...")
response = send_mcp_request("tools/call", {
    "name": "list_processes",
    "arguments": {}
})
print(f"Response: {response}")

# Test 3: Get status
print("\n3. Getting status...")
response = send_mcp_request("tools/call", {
    "name": "get_status",
    "arguments": {}
})
print(f"Response: {response}")
