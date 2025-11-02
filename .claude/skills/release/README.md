# Release Skill

Vantage MCPプロジェクトのリリースプロセスを管理するためのスキル。

## 📁 ファイル構成

```
release/
├── SKILL.md          # スキル本体のドキュメント
├── README.md         # このファイル
└── scripts/
    ├── types.ts      # TypeScript型定義
    ├── lib.ts        # ユーティリティ関数
    ├── release.ts    # メインリリーススクリプト
    └── tsconfig.json # TypeScript設定
```

## 🎯 目的

このスキルは以下をサポートします：

1. **バージョン管理** - セマンティックバージョニングに従ったバージョン管理
2. **リリース前チェック** - ビルド、テスト、CIチェックの確認
3. **リリース作成** - Gitタグ作成とGitHubリリースの公開
4. **自動ロールバック** - テストやビルドが失敗した場合の自動復元

## 🚀 クイックスタート

### 必要なツール

- **Bun**: TypeScriptランタイム
  ```bash
  curl -fsSL https://bun.sh/install | bash
  ```

- **gh CLI**: GitHubリリース作成用
  ```bash
  brew install gh
  ```

### リリース手順

```bash
# 1. scriptsディレクトリに移動
cd .claude/skills/release/scripts

# 2. リリーススクリプトを実行
bun run release.ts 0.1.0-beta21

# 3. 画面の指示に従う
# - 事前チェック結果を確認
# - バージョン情報を確認
# - 確認プロンプトで "yes" を入力
# - テストとビルドが自動実行される
# - プッシュ確認で "yes" を入力

# 4. GitHubリリースを作成（スクリプトが表示するコマンドを実行）
gh release create v0.1.0-beta21 \
  --title "v0.1.0-beta21 - タイトル" \
  --notes-file release-notes.md \
  --prerelease
```

## 📋 リリースチェックリスト

### 事前準備

- [ ] Bunがインストールされている（`bun --version`で確認）
- [ ] mainブランチにいる
- [ ] リモートと同期している（`git pull origin main`）
- [ ] 未コミットの変更がない（`git status`で確認）
- [ ] CIチェックが通過している
- [ ] レビュー承認を取得している

### リリースノート作成

- [ ] `release-notes.md`を作成
- [ ] 新機能、バグ修正、改善点を記載
- [ ] Breaking Changesがあれば記載
- [ ] インストール方法を記載

### リリース実行

- [ ] リリーススクリプトを実行
- [ ] 事前チェックがすべて通過
- [ ] テストが成功
- [ ] ビルドが成功
- [ ] タグが作成された
- [ ] リモートにプッシュされた
- [ ] GitHubリリースを作成

## 🔧 スクリプト詳細

### release.ts

メインリリーススクリプト。以下の8ステップを自動実行します：

1. **�� 事前チェック**
   - mainブランチにいるか確認
   - 未コミット変更がないか確認
   - リモートと同期しているか確認

2. **📝 バージョン情報の表示**
   - 現在のバージョンと新しいバージョンを表示
   - ユーザー確認を求める

3. **🔧 Cargo.toml と Cargo.lock の更新**
   - バージョンフィールドを書き換え
   - `cargo build`でCargo.lockを更新

4. **🧪 テストの実行**
   - `cargo test`を実行
   - 失敗時は自動ロールバック

5. **🔨 リリースビルド**
   - `cargo build --release`を実行
   - 失敗時は自動ロールバック

6. **📦 Gitコミットの作成**
   - Cargo.tomlとCargo.lockをコミット
   - コミットメッセージ: `chore: bump version to vX.Y.Z`

7. **🏷️ Gitタグの作成**
   - アノテーテッドタグを作成
   - タグメッセージ: `Release vX.Y.Z`

8. **🚢 リモートへのプッシュ（確認付き）**
   - プッシュ前にユーザー確認
   - ブランチとタグを同時にプッシュ

**使用例:**
```bash
cd .claude/skills/release/scripts
bun run release.ts 0.1.0-beta21
```

### lib.ts

ユーティリティ関数ライブラリ：

- **Version Utils**: バージョンのパース、フォーマット、タグ変換
- **Cargo Utils**: Cargo.toml読み書き、ビルド、テスト実行
- **Git Utils**: ブランチ確認、コミット作成、タグ作成、プッシュ
- **Pre-flight Checks**: リリース前の包括的チェック

### types.ts

TypeScript型定義：

- `Version`: バージョン情報（major, minor, patch, prerelease）
- `ReleaseConfig`: リリース設定（currentVersion, newVersion, tag）
- `CheckResult`: チェック結果（passed, error）

## 🔄 ロールバック機能

スクリプトは以下の場合に自動的にロールバックします：

- **テスト失敗時**: Cargo.tomlを元のバージョンに戻し、Cargo.lockを再生成
- **ビルド失敗時**: 同様にロールバック
- **タグ作成失敗時**: コミットを取り消し、ファイルをロールバック

手動でのクリーンアップは不要です。

## 📖 参考資料

詳細なドキュメントは `SKILL.md` を参照してください。

- バージョニング規則
- リリース手順の詳細
- ベストプラクティス
- トラブルシューティング

## 🔗 関連ドキュメント

- [SKILL.md](SKILL.md) - スキル本体のドキュメント
- [CLAUDE.md](../../CLAUDE.md) - プロジェクト全体のガイド
- [BRANCH_PROTECTION.md](../../../.github/BRANCH_PROTECTION.md) - ブランチ保護ルール

## 🛠️ 開発者向け情報

### スクリプトの修正

スクリプトを修正する場合：

1. `types.ts`: 型定義を変更
2. `lib.ts`: ユーティリティ関数を追加・修正
3. `release.ts`: メインフローを変更
4. `tsconfig.json`: TypeScript設定を調整

### TypeScript設定

- **ターゲット**: ESNext
- **モジュールシステム**: ESNext (Bunのbundlerモード)
- **厳格モード**: 有効
- **Bunタイプ**: 有効

### テスト方法

```bash
# ドライラン（テストのみ実行、コミット・プッシュはしない）
# 手動で各関数をインポートしてテスト可能
bun run -e 'import { parseVersion } from "./lib.ts"; console.log(parseVersion("0.1.0-beta21"))'
```

## 📝 更新履歴

- 2025-01-02: 初版作成（シェルスクリプト版）
- 2025-01-02: Bun + TypeScript版に全面移行、自動ロールバック機能追加

## 💡 Tips

### プッシュをスキップした場合

```bash
# 後でプッシュする
git push origin main
git push origin v0.1.0-beta21
```

### タグを削除したい場合

```bash
# ローカルタグを削除
git tag -d v0.1.0-beta21

# リモートタグを削除
git push origin :refs/tags/v0.1.0-beta21
```

### リリースノートのテンプレート

```markdown
# v0.1.0-betaXX

## 🎉 新機能
- 機能A: 説明

## 🐛 バグ修正
- 修正A: 説明

## 🔧 改善
- 改善A: 説明

## 📚 ドキュメント
- ドキュメントA: 説明

## インストール方法

```bash
cargo install --git https://github.com/chronista-club/vantage-mcp --tag v0.1.0-betaXX vantage-mcp
```
```
