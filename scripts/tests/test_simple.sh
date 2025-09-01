#!/bin/bash

# シンプルなテストスクリプト

PORT=12700
API="http://localhost:$PORT/api"

echo "=== Simple Auto-Start Test ==="
echo ""

# 1. auto_start_on_createのテスト
echo "1. Testing auto_start_on_create..."
echo "Creating process that should start immediately..."
curl -X POST $API/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "immediate-start",
    "command": "sleep",
    "args": ["3"],
    "auto_start_on_create": true,
    "auto_start_on_restore": false
  }'
echo ""
sleep 1

echo "Process list:"
curl -s $API/processes | jq -r '.[] | "\(.id): \(.state.state) (on_create=\(.auto_start_on_create), on_restore=\(.auto_start_on_restore))"'
echo ""

# 2. auto_start_on_restoreのテスト
echo "2. Testing auto_start_on_restore..."
echo "Creating process that should NOT start now..."
curl -X POST $API/processes \
  -H "Content-Type: application/json" \
  -d '{
    "id": "restore-only",
    "command": "sleep",
    "args": ["3"],
    "auto_start_on_create": false,
    "auto_start_on_restore": true
  }'
echo ""
sleep 1

echo "Process list:"
curl -s $API/processes | jq -r '.[] | "\(.id): \(.state.state) (on_create=\(.auto_start_on_create), on_restore=\(.auto_start_on_restore))"'
echo ""

echo "=== Summary ==="
echo "immediate-start should be Running (auto_start_on_create=true)"
echo "restore-only should be NotStarted (auto_start_on_restore=true but not started yet)"