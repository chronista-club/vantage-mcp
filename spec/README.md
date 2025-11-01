# Vantage MCP 仕様書

このディレクトリには、Vantage MCP プロジェクトの詳細な仕様書が格納されています。

## 📋 仕様書の構成

### 📄 pages/ - ページ仕様

各ページの目的、機能、UI/UX要件を定義します。

- [processes.md](./pages/processes.md) - プロセス管理ページ
- [templates.md](./pages/templates.md) - テンプレート管理ページ
- [clipboard.md](./pages/clipboard.md) - クリップボード管理ページ
- [settings.md](./pages/settings.md) - 設定ページ
- [dashboard.md](./pages/dashboard.md) - ダッシュボード（将来実装予定）

### ⚙️ processes/ - プロセス管理仕様

プロセスのライフサイクル、起動・終了動作、状態管理に関する仕様を定義します。

- [lifecycle.md](./processes/lifecycle.md) - プロセスライフサイクルとステートマシン
- [auto-start.md](./processes/auto-start.md) - 自動起動の仕様
- [shutdown.md](./processes/shutdown.md) - 終了時の仕様
- [monitoring.md](./processes/monitoring.md) - 監視とヘルスチェック

### 🏗️ architecture/ - アーキテクチャ仕様

システム全体の設計、ディレクトリ構成、技術スタックを定義します。

- [directory-structure.md](./architecture/directory-structure.md) - ディレクトリ構成とファイル配置
- [3d-ui-system.md](./architecture/3d-ui-system.md) - 3D UI システムアーキテクチャ
- [state-management.md](./architecture/state-management.md) - 状態管理（Pinia）
- [theme-system.md](./architecture/theme-system.md) - テーマシステム（OKLCH）

## 🎯 仕様書の目的

### 1. **開発の統一性**
全ての開発者が同じ理解のもとで開発を進められるよう、明確な仕様を提供します。

### 2. **品質保証**
機能要件、非機能要件、エッジケースを事前に定義することで、実装の品質を担保します。

### 3. **保守性の向上**
将来の機能追加や改修時に、既存の設計意図を理解できるようドキュメント化します。

### 4. **Claude Code との連携**
AI アシスタントが仕様を参照し、一貫性のある実装を支援できるようにします。

## 📐 仕様書作成ガイドライン

### 必須項目

各仕様書には以下を含めること：

1. **目的** - なぜこの機能/ページが必要か
2. **ユースケース** - 誰がどのように使うか
3. **要件** - 機能要件と非機能要件
4. **制約** - 技術的制約、ビジネス制約
5. **設計** - 具体的な実装方針
6. **テストケース** - 検証すべき項目

### 推奨フォーマット

```markdown
# [機能名]

## 目的
この機能の存在理由と解決する課題

## ユースケース
- ユーザーストーリー形式で記述
- 「〜として、〜したい、なぜなら〜」

## 機能要件
- FR-001: 必須機能
- FR-002: オプション機能

## 非機能要件
- NFR-001: パフォーマンス要件
- NFR-002: セキュリティ要件

## 設計
具体的な実装方針、使用技術

## テストケース
- TC-001: 正常系
- TC-002: 異常系
```

## 🔄 更新履歴

仕様変更時は必ず以下を記録：

- 変更日
- 変更者
- 変更内容
- 変更理由

## 📚 関連ドキュメント

- [CLAUDE.md](../CLAUDE.md) - プロジェクト全体のガイダンス
- [README.md](../README.md) - プロジェクト概要
- [CHANGELOG.md](../CHANGELOG.md) - 変更履歴
- [Projects/WebXR/](../Projects/WebXR/) - 3D UI 関連の技術調査資料（Obsidian）

## 🤝 コントリビューション

仕様書の改善提案は、以下の流れで行ってください：

1. Issue で提案を議論
2. 合意が得られたら PR を作成
3. レビュー承認後にマージ

---

**Note**: この仕様書は生きたドキュメントです。実装と並行して継続的に更新し、常に最新の状態を保ちます。
