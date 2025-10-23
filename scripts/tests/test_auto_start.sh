#!/bin/bash

# Vantageサーバーのテストスクリプト
# auto_start_on_createとauto_start_on_restoreフラグのテスト

API_URL="http://localhost:12700/api"
# ポートが12701の場合
if ! curl -s -o /dev/null -w "%{http_code}" $API_URL/status | grep -q "200"; then
    API_URL="http://localhost:12701/api"
fi

echo "=== Vantage Auto-Start Flags Test ==="
echo "API URL: $API_URL"
echo ""

# テスト1: auto_start_on_createフラグのテスト
echo "Test 1: auto_start_on_create flag"
echo "Creating process with auto_start_on_create=true..."
curl -s -X POST $API_URL/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test1-auto-create",
    "command": "sleep",
    "args": ["5"],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }'
echo ""
sleep 1

echo "Checking process status..."
STATUS=$(curl -s $API_URL/processes/test1-auto-create | jq -r '.state.state')
echo "Status: $STATUS"
if [ "$STATUS" = "Running" ]; then
    echo "✓ Test 1 PASSED: Process auto-started on creation"
else
    echo "✗ Test 1 FAILED: Process did not auto-start (status: $STATUS)"
fi
echo ""

# テスト2: auto_start_on_restoreフラグのテスト
echo "Test 2: auto_start_on_restore flag"
echo "Creating process with auto_start_on_restore=true (but not auto_start_on_create)..."
curl -s -X POST $API_URL/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test2-auto-restore",
    "command": "sleep",
    "args": ["10"],
    "auto_start_on_create": false,
    "auto_start_on_restore": true
  }'
echo ""
sleep 1

echo "Checking process status (should NOT be running)..."
STATUS=$(curl -s $API_URL/processes/test2-auto-restore | jq -r '.state.state')
echo "Status: $STATUS"
if [ "$STATUS" = "NotStarted" ]; then
    echo "✓ Test 2a PASSED: Process did not auto-start on creation"
else
    echo "✗ Test 2a FAILED: Process should not have started (status: $STATUS)"
fi
echo ""

# テスト3: 両方のフラグがtrueの場合
echo "Test 3: Both flags true"
echo "Creating process with both flags true..."
curl -s -X POST $API_URL/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test3-both-flags",
    "command": "sleep",
    "args": ["7"],
    "auto_start_on_create": true,
    "auto_start_on_restore": true
  }'
echo ""
sleep 1

echo "Checking process status..."
STATUS=$(curl -s $API_URL/processes/test3-both-flags | jq -r '.state.state')
echo "Status: $STATUS"
if [ "$STATUS" = "Running" ]; then
    echo "✓ Test 3 PASSED: Process auto-started with both flags"
else
    echo "✗ Test 3 FAILED: Process did not auto-start (status: $STATUS)"
fi
echo ""

# 全プロセスのリスト表示
echo "=== All Processes ==="
curl -s $API_URL/processes | jq '.[] | {id: .id, state: .state.state, auto_start: .auto_start, auto_start_on_create: .auto_start_on_create, auto_start_on_restore: .auto_start_on_restore}'
echo ""

# エクスポートしてファイル確認
echo "=== Exporting processes for persistence test ==="
curl -s -X POST $API_URL/export \
  -H "Content-Type: application/json" \
  -d '{"file_path": "/tmp/vantage_test_export.surql"}'
echo ""

# クリーンアップ
echo "=== Cleanup ==="
echo "Removing test processes..."
curl -s -X DELETE $API_URL/processes/test1-auto-create
curl -s -X DELETE $API_URL/processes/test2-auto-restore
curl -s -X DELETE $API_URL/processes/test3-both-flags
echo "Test completed!"