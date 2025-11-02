# リリーススキル

Vantage MCPプロジェクトのリリースプロセスを自動化・サポートするスキル。

## 概要

このスキルは、バージョン管理からGitHubリリースの作成まで、リリースプロセス全体を自動化します。
Bun + TypeScriptで実装された決定論的なスクリプトにより、一貫性のあるリリースを保証します。

## 主な機能

### 1. バージョン管理
- Cargo.tomlのバージョン更新
- セマンティックバージョニングのサポート
- バージョン番号の一貫性チェック

### 2. リリース前チェック
- ビルドの成功確認
- テストの実行と検証
- ブランチ保護ルールの遵守確認
- 未コミット変更の検出
- リモート同期の確認

### 3. リリース作成
- Gitタグの自動作成とプッシュ
- GitHubリリースの作成サポート
- ロールバック機能（テスト失敗時）

### 4. リリース後処理
- 次期バージョンへの準備
- ドキュメントの更新

## 使用方法

### リリーススクリプトの実行

```bash
# scripts ディレクトリに移動
cd .claude/skills/release/scripts

# リリーススクリプトを実行
bun run release.ts <version>

# 例: Beta版のリリース
bun run release.ts 0.1.0-beta21

# 例: 正式版のリリース
bun run release.ts 0.2.0
```

### リリーススクリプトの処理フロー

リリーススクリプトは以下の8ステップを自動実行します：

1. **📋 事前チェック**
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

9. **🎉 次のステップの表示**
   - GitHubリリース作成コマンドを表示
   - インストール方法を表示

### エラーハンドリングとロールバック

スクリプトは以下の場合に自動的にロールバックします：

- **テスト失敗時**: Cargo.tomlを元のバージョンに戻し、Cargo.lockを再生成
- **ビルド失敗時**: 同様にロールバック
- **タグ作成失敗時**: コミットを取り消し、ファイルをロールバック

## リリース手順の詳細

### ステップ1: 事前準備

1. **ブランチの確認**
   ```bash
   git status
   git branch
   ```

2. **最新の変更を取得**
   ```bash
   git pull origin main
   ```

3. **CIチェックの確認**
   - GitHubのPRやActionsで全てのチェックが通過していることを確認

### ステップ2: リリースノート作成

リリース前に`release-notes.md`を作成します：

```markdown
# v0.1.0-betaXX

## 🎉 新機能
- 機能A: 説明
- 機能B: 説明

## 🐛 バグ修正
- 修正A: 説明
- 修正B: 説明

## 🔧 改善
- 改善A: 説明
- 改善B: 説明

## 📚 ドキュメント
- ドキュメントA: 説明

## ⚠️ Breaking Changes
- 変更A: 説明（該当する場合）

## インストール方法

```bash
cargo install --git https://github.com/chronista-club/vantage-mcp --tag v0.1.0-betaXX vantage-mcp
```
```

### ステップ3: リリーススクリプト実行

```bash
cd .claude/skills/release/scripts
bun run release.ts 0.1.0-beta21
```

スクリプトが以下を自動実行します：
- 事前チェック
- Cargo.toml/Cargo.lock更新
- テストとビルド
- Gitコミットとタグ作成
- リモートプッシュ（確認後）

### ステップ4: GitHubリリース作成

スクリプトが表示するコマンドを実行：

```bash
gh release create v0.1.0-beta21 \
  --title "v0.1.0-beta21 - リリースタイトル" \
  --notes-file release-notes.md \
  --prerelease
```

## ベストプラクティス

### 1. バージョニング規則

- **Beta版**: `0.1.0-beta1`, `0.1.0-beta2`, ...
- **RC版**: `0.1.0-rc1`, `0.1.0-rc2`, ...
- **正式版**: `0.1.0`, `0.2.0`, `1.0.0`, ...

### 2. タグ命名規則

- 必ず`v`プレフィックスを付ける
- `v0.1.0-beta20`のような形式
- Cargo.tomlのバージョンと完全に一致させる

### 3. リリースノート

- 変更内容を明確に記述
- ユーザーへの影響を説明
- Breaking Changesは必ず記載
- インストール方法を含める

### 4. プレリリースとリリース

- 開発中は`--prerelease`フラグを使用
- 正式版では`--latest`フラグを使用（または省略）

## トラブルシューティング

### バージョンが一致しない

```bash
# Cargo.tomlのバージョンを確認
grep "^version" Cargo.toml

# タグ一覧を確認
git tag -l

# 不要なタグを削除
git tag -d v0.1.0-betaXX
git push origin :refs/tags/v0.1.0-betaXX
```

### テストやビルドが失敗する

リリーススクリプトは自動的にロールバックするため、手動での修正は不要です。
失敗の原因を修正してから再度スクリプトを実行してください。

### cargo installが失敗する

- タグ名が正しいか確認（`v`プレフィックス必須）
- Cargo.tomlのバージョンとタグが一致しているか確認
- リリースが公開されているか確認

### プッシュをキャンセルした場合

```bash
# 後でプッシュする場合
git push origin main
git push origin v0.1.0-beta21
```

## スクリプト構成

### ファイル構成

```
.claude/skills/release/scripts/
├── types.ts         # 型定義
├── lib.ts           # ユーティリティ関数
├── release.ts       # メインスクリプト
└── tsconfig.json    # TypeScript設定
```

### 型定義（types.ts）

- `Version`: バージョン情報（major, minor, patch, prerelease）
- `ReleaseConfig`: リリース設定（currentVersion, newVersion, tag）
- `CheckResult`: チェック結果（passed, error）

### ユーティリティ関数（lib.ts）

- **Version Utils**: `parseVersion()`, `formatVersion()`, `formatTag()`
- **Cargo Utils**: `getCurrentVersion()`, `updateCargoToml()`, `updateCargoLock()`, `runCargoTest()`, `runCargoBuildRelease()`
- **Git Utils**: `getCurrentBranch()`, `hasUncommittedChanges()`, `isSyncedWithRemote()`, `createCommit()`, `createTag()`, `pushToRemote()`
- **Pre-flight Checks**: `checkPrerequisites()`

## チェックリスト

リリース前に以下を確認してください：

- [ ] mainブランチにいる
- [ ] リモートと同期している
- [ ] 未コミットの変更がない
- [ ] CIチェックがすべて通過している
- [ ] 必要なレビュー承認を取得している
- [ ] リリースノート（release-notes.md）を作成した
- [ ] Bunがインストールされている
- [ ] プレリリースか正式版かを決定した

## 参考資料

- [セマンティックバージョニング](https://semver.org/lang/ja/)
- [GitHub CLI - リリース作成](https://cli.github.com/manual/gh_release_create)
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Bun Documentation](https://bun.sh/docs)

## 更新履歴

- 2025-01-02: 初版作成（シェルスクリプト版）
- 2025-01-02: Bun + TypeScript版に移行、自動ロールバック機能追加
