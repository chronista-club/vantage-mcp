# Ichimi Server ドキュメント

Ichimi Server の公式ドキュメントへようこそ。

## 📚 ドキュメント構成

### [アーキテクチャ](./architecture/)
- [システム概要](./architecture/README.md) - アーキテクチャと設計原則

### [APIリファレンス](./api/)
- [MCP ツール API](./api/README.md#mcp-ツール-api) - Claude Code から使用可能なツール
- [REST API](./api/README.md#rest-api) - Webダッシュボード用API

### [ガイド](./guides/)
- [インストールガイド](./guides/installation.md) - セットアップと設定
- [使用ガイド](./guides/usage.md) - 基本的な使い方と応用例

## 🚀 クイックスタート

### 1. インストール

```bash
cargo install --git https://github.com/chronista-club/ichimi-server --tag v0.2.0
```

### 2. Claude Code 設定

`~/.config/claude/mcp.json`:
```json
{
  "mcpServers": {
    "ichimi": {
      "command": "ichimi",
      "args": []
    }
  }
}
```

### 3. 基本的な使用例

Claude Code で：
```
Create a process called "hello" that runs "echo Hello, World!"
Start the hello process
```

## 🎯 主な機能

- **プロセス管理**: プロセスのライフサイクル全体を管理
- **リアルタイム監視**: stdout/stderr のリアルタイムキャプチャ
- **CI/CD統合**: GitHub Actions の監視と制御
- **Webダッシュボード**: ブラウザベースの管理UI
- **永続化**: プロセス設定の保存と復元

## 📦 バージョン情報

- **現在のバージョン**: v0.2.0
- **最小Rustバージョン**: 1.75

## 🔄 更新履歴

### v0.2.0 (2025-09-28)
- SurrealDB依存関係を削除
- インメモリストレージへ移行
- パフォーマンスと安定性の向上

### v0.1.0-beta20
- 初期リリース
- 基本的なプロセス管理機能
- MCP統合

## 🤝 コントリビューション

Issues や Pull Requests は [GitHub リポジトリ](https://github.com/chronista-club/ichimi-server) で歓迎します。

## 📄 ライセンス

MIT OR Apache-2.0

## 🔗 関連リンク

- [GitHub リポジトリ](https://github.com/chronista-club/ichimi-server)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [Claude Code](https://claude.ai/code)