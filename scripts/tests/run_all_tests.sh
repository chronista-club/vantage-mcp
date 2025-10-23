#!/bin/bash

# Vantage Server 統合テストスイート
# すべてのテストを順番に実行

set -e  # エラーが発生したら停止

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

# カラー出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# テスト結果を記録
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# テスト実行関数
run_test() {
    local test_name=$1
    local test_file=$2
    
    echo -e "\n${YELLOW}Running test: $test_name${NC}"
    echo "================================================"
    
    if bash "$test_file"; then
        echo -e "${GREEN}✓ $test_name passed${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ $test_name failed${NC}"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("$test_name")
    fi
    
    # クリーンアップ
    pkill -f "vantagemcp.*--web" 2>/dev/null || true
    pkill -f "sleep 9999" 2>/dev/null || true
    rm -f /tmp/test_*.txt /tmp/test_*.sh 2>/dev/null || true
    sleep 1
}

# ビルド
echo -e "${YELLOW}Building vantage...${NC}"
cargo build --release

# テスト実行
run_test "Simple Process Test" "$SCRIPT_DIR/test_simple.sh"
run_test "Auto-Start Test" "$SCRIPT_DIR/test_auto_start.sh"
run_test "Shutdown Behavior Test" "$SCRIPT_DIR/test_shutdown.sh"
run_test "Database Persistence Test" "$SCRIPT_DIR/test_db_persistence.sh"
run_test "MCP Protocol Test" "$SCRIPT_DIR/test_mcp_protocol.sh"

# 結果サマリー
echo -e "\n================================================"
echo -e "${YELLOW}Test Results Summary${NC}"
echo -e "================================================"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo -e "\n${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  - $test"
    done
    exit 1
else
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
fi