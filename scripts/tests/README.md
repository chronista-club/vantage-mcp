# Vantage MCP テストスイート

## テストの実行方法

### すべてのテストを実行
```bash
./scripts/tests/run_all_tests.sh
```

### 個別のテストを実行
```bash
# シンプルなプロセステスト
./scripts/tests/test_simple.sh

# 自動起動テスト
./scripts/tests/test_auto_start.sh

# シャットダウン動作テスト
./scripts/tests/test_shutdown.sh

# データベース永続化テスト
./scripts/tests/test_db_persistence.sh

# MCPプロトコルテスト（要: @modelcontextprotocol/sdk）
./scripts/tests/test_mcp_protocol.sh
```

## テスト内容

| テストファイル | 説明 | テスト内容 |
|---|---|---|
| `test_simple.sh` | 基本動作テスト | プロセスの作成、起動、停止の基本操作 |
| `test_auto_start.sh` | 自動起動テスト | auto_start_on_restoreフラグの動作確認 |
| `test_shutdown.sh` | シャットダウンテスト | サーバー終了時のプロセス継続/停止動作 |
| `test_db_persistence.sh` | 永続化テスト | SurrealDBへのエクスポート/インポート機能 |
| `test_mcp_protocol.sh` | MCPプロトコルテスト | MCPツールの呼び出しとレスポンス確認 |

## 注意事項

- テストはすべて`cargo build --release`でビルドされたバイナリを使用
- テスト実行後は自動的にクリーンアップ処理が実行される
- ポート番号は各テストで異なるポートを使用（競合を避けるため）
  - test_simple: デフォルトポート
  - test_auto_start: デフォルトポート
  - test_shutdown: デフォルトポート
  - test_db_persistence: 12720-12721
  - test_mcp_protocol: MCPソケット通信

## トラブルシューティング

### プロセスが残ってしまった場合
```bash
pkill -f vantage
pkill -f "sleep 9999"
```

### テストファイルが残ってしまった場合
```bash
rm -f /tmp/test_*.txt /tmp/test_*.sh
rm -rf /tmp/vantage_test_*
```