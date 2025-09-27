#!/bin/bash

set -e

echo "=== Simple Graceful Shutdown Test ==="

# カラー定義
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# テスト用のTypeScriptスクリプトを作成
cat > /tmp/test-graceful.ts << 'EOF'
let isShuttingDown = false;

process.on("SIGTERM", async () => {
  if (isShuttingDown) return;
  isShuttingDown = true;
  console.log("📍 Received SIGTERM, starting graceful shutdown...");

  for (let i = 1; i <= 3; i++) {
    console.log(`  Cleanup step ${i}/3...`);
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  console.log("✅ Graceful shutdown complete");
  process.exit(0);
});

console.log(`🚀 Process started with PID ${process.pid}`);
setInterval(() => {
  if (!isShuttingDown) {
    console.log("💼 Working...");
  }
}, 2000);
EOF

# テストプロセスを直接起動
echo -e "${YELLOW}Starting test process...${NC}"
bun run /tmp/test-graceful.ts &
TEST_PID=$!

echo "Process started with PID: $TEST_PID"
sleep 3

# SIGTERMを送信
echo -e "${YELLOW}Sending SIGTERM to process...${NC}"
kill -TERM $TEST_PID

# グレースフルシャットダウンを待つ
echo -e "${YELLOW}Waiting for graceful shutdown...${NC}"
sleep 2

# プロセスが終了しているか確認
if kill -0 $TEST_PID 2>/dev/null; then
  echo -e "${RED}❌ Process is still running after SIGTERM${NC}"
  kill -KILL $TEST_PID 2>/dev/null
else
  echo -e "${GREEN}✅ Process terminated gracefully${NC}"
fi

# クリーンアップ
rm -f /tmp/test-graceful.ts

echo -e "${GREEN}Test complete!${NC}"