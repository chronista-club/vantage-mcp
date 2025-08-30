#!/bin/bash

# データベース直接テストスクリプト

echo "=== Direct Database Test ==="
echo ""

# 1. SurrealDBに直接データを作成
echo "1. Creating test data directly in SurrealDB..."
cat > /tmp/test_import.surql << 'EOF'
CREATE process CONTENT {
    process_id: "direct-test-1",
    command: "ls",
    args: ["-la"],
    env: {},
    cwd: "/tmp",
    auto_start_on_create: false,
    auto_start_on_restore: true,
    updated_at: time::now()
};

CREATE process CONTENT {
    process_id: "direct-test-2",
    command: "echo",
    args: ["hello"],
    env: {},
    cwd: null,
    auto_start_on_create: true,
    auto_start_on_restore: false,
    updated_at: time::now()
};
EOF

echo "Test data created in /tmp/test_import.surql"
echo ""

# 2. Rustテストコードを実行してエクスポート機能をテスト
echo "2. Running export test..."
cargo test --test test_surrealdb_persistence test_database_export_import -- --nocapture

echo ""
echo "=== Test Complete ==="