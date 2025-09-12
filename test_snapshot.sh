#!/bin/bash

# Test snapshot functionality

echo "=== Testing Snapshot Functionality ==="

# 1. Add some test data
echo "1. Adding test data..."
curl -s -X POST http://localhost:12701/api/clipboard/text \
  -H "Content-Type: application/json" \
  -d '{"content":"Test data 1","tags":["test","data1"]}' | jq -r '.id'

curl -s -X POST http://localhost:12701/api/clipboard/text \
  -H "Content-Type: application/json" \
  -d '{"content":"Test data 2","tags":["test","data2"]}' | jq -r '.id'

# 2. Check current clipboard items
echo "2. Current clipboard items:"
curl -s -X GET "http://localhost:12701/api/clipboard/history?limit=10" | jq '.items[].content'

# 3. Export to surql file
echo "3. Creating snapshot..."
# We need to trigger export through the ProcessManager's database
# For now, let's manually export using the API

# 4. Clear clipboard
echo "4. Clearing clipboard..."
curl -s -X DELETE http://localhost:12701/api/clipboard

# 5. Verify clipboard is empty
echo "5. Clipboard after clear:"
curl -s -X GET "http://localhost:12701/api/clipboard/history?limit=10" | jq '.items'

# 6. Import from snapshot
echo "6. TODO: Restore from snapshot (need to implement API endpoint)"

echo "=== Test Complete ==="