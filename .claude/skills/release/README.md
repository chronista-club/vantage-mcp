# Release Skill

Vantage MCPプロジェクトのリリースプロセスを管理するためのスキル。

## 📁 ファイル構成

```
release/
├── SKILL.md                          # スキル本体のドキュメント
├── README.md                         # このファイル
└── scripts/
    ├── release.sh                    # リリース自動化スクリプト
    └── generate-release-notes.sh     # リリースノート生成スクリプト
```

## 🎯 目的

このスキルは以下をサポートします：

1. **バージョン管理** - セマンティックバージョニングに従ったバージョン管理
2. **リリース前チェック** - ビルド、テスト、CIチェックの確認
3. **リリース作成** - Gitタグ作成とGitHubリリースの公開
4. **リリースノート生成** - コミット履歴からの自動生成

## 🚀 クイックスタート

### 方法1: 対話的に実行

Claude Codeでスキルを起動：

```
このプロジェクトをリリースしたいです。v0.1.0-beta21をリリースしてください。
```

### 方法2: スクリプトを使用

```bash
# 1. リリースノート生成
./.claude/skills/release/scripts/generate-release-notes.sh v0.1.0-beta20 0.1.0-beta21

# 2. リリースノートを編集
vim release-notes.md

# 3. リリース実行
./.claude/skills/release/scripts/release.sh 0.1.0-beta21

# 4. GitHubリリース作成
gh release create v0.1.0-beta21 \
  --title "v0.1.0-beta21 - タイトル" \
  --notes-file release-notes.md \
  --prerelease
```

## 📋 リリースチェックリスト

- [ ] mainブランチにいる
- [ ] リモートと同期している
- [ ] 未コミットの変更がない
- [ ] CIチェックが通過している
- [ ] レビュー承認を取得している
- [ ] Cargo.tomlのバージョンを更新した
- [ ] リリースノートを作成した
- [ ] ビルドとテストが成功する

## 🔧 スクリプト詳細

### release.sh

リリースプロセス全体を自動化します：

1. 事前確認（ブランチ、同期状態、未コミット変更）
2. Cargo.tomlのバージョン更新
3. Cargo.lockの更新
4. ビルドとテスト
5. コミットとタグ作成
6. リモートへのプッシュ（確認付き）

**使用例:**
```bash
./.claude/skills/release/scripts/release.sh 0.1.0-beta21
```

### generate-release-notes.sh

コミット履歴からリリースノートを自動生成します：

- 新機能（feat:）
- バグ修正（fix:）
- 改善・リファクタリング（refactor:, perf:）
- ドキュメント（docs:）
- スタイル・UI改善（style:）
- 全コミットリスト
- 貢献者リスト

**使用例:**
```bash
./.claude/skills/release/scripts/generate-release-notes.sh v0.1.0-beta20 0.1.0-beta21
```

## 📖 参考資料

詳細なドキュメントは `SKILL.md` を参照してください。

- バージョニング規則
- リリース手順の詳細
- ベストプラクティス
- トラブルシューティング

## 🔗 関連ドキュメント

- [CLAUDE.md](../../CLAUDE.md) - プロジェクト全体のガイド
- [BRANCH_PROTECTION.md](../../../.github/BRANCH_PROTECTION.md) - ブランチ保護ルール

## 📝 更新履歴

- 2025-01-02: 初版作成
