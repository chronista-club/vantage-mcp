# リリーススキル

Vantage MCPプロジェクトのリリースプロセスを自動化・サポートするスキル。

## 概要

このスキルは、バージョン管理からGitHubリリースの作成まで、リリースプロセス全体をガイドします。

## 主な機能

### 1. バージョン管理
- Cargo.tomlのバージョン更新
- セマンティックバージョニングのサポート
- バージョン番号の一貫性チェック

### 2. リリース前チェック
- ビルドの成功確認
- テストの実行と検証
- CIチェックの確認
- ブランチ保護ルールの遵守確認

### 3. リリース作成
- Gitタグの作成とプッシュ
- GitHubリリースの作成
- リリースノートの生成

### 4. リリース後処理
- 次期バージョンへの準備
- ドキュメントの更新

## 使用方法

### 基本的なリリースフロー

```bash
# 1. リリーススキルを起動
/release

# 2. 対話的にリリースプロセスを進める
# - 現在のバージョンを確認
# - 新しいバージョンを入力
# - リリースノートを作成
# - リリースを実行
```

## リリース手順の詳細

### ステップ1: 事前確認

リリース前に以下を確認します：

1. **ブランチの状態**
   - mainブランチにいること
   - リモートと同期していること
   - 未コミットの変更がないこと

2. **CIチェック**
   - すべてのテストが通過していること
   - Clippyで警告がないこと
   - Rustfmtでフォーマットされていること

3. **ブランチ保護**
   - 必要なレビュー承認を取得していること
   - すべての会話が解決されていること

### ステップ2: バージョン更新

1. **現在のバージョンを確認**
   ```bash
   grep "^version" Cargo.toml
   ```

2. **新しいバージョンを決定**
   - セマンティックバージョニングに従う
   - MAJOR.MINOR.PATCH[-PRERELEASE]
   - 例: `0.1.0-beta20` → `0.1.0-beta21`

3. **Cargo.tomlを更新**
   ```bash
   # versionフィールドを更新
   # 例: version = "0.1.0-beta20" → version = "0.1.0-beta21"
   ```

4. **Cargo.lockを更新**
   ```bash
   cargo build
   ```

### ステップ3: ビルドとテスト

```bash
# リリースビルド
cargo build --release

# テスト実行
cargo test

# Clippy実行
cargo clippy -- -D warnings

# フォーマットチェック
cargo fmt -- --check
```

### ステップ4: リリースノート作成

リリースノートには以下を含めます：

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

### ステップ5: コミットとタグ作成

```bash
# 1. 変更をステージング
git add Cargo.toml Cargo.lock

# 2. コミット作成
git commit -m "chore: bump version to v0.1.0-betaXX"

# 3. タグ作成
git tag -a v0.1.0-betaXX -m "Release v0.1.0-betaXX - 簡潔な説明"

# 4. プッシュ
git push origin main
git push origin v0.1.0-betaXX
```

### ステップ6: GitHubリリース作成

```bash
# リリースノートファイルを使用
gh release create v0.1.0-betaXX \
  --title "v0.1.0-betaXX - リリースタイトル" \
  --notes-file release-notes.md \
  --prerelease

# または、インラインでノートを指定
gh release create v0.1.0-betaXX \
  --title "v0.1.0-betaXX - リリースタイトル" \
  --notes "リリースノート本文" \
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
- 正式版では`--latest`フラグを使用

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

### cargo installが失敗する

- タグ名が正しいか確認（`v`プレフィックス必須）
- Cargo.tomlのバージョンとタグが一致しているか確認
- リリースが公開されているか確認

### CIチェックが失敗している

- mainブランチでCIが通過していることを確認
- 必要に応じてPRを作成して修正
- すべてのチェックが通過してからリリース

## チェックリスト

リリース前に以下を確認してください：

- [ ] mainブランチにいる
- [ ] リモートと同期している
- [ ] 未コミットの変更がない
- [ ] CIチェックがすべて通過している
- [ ] 必要なレビュー承認を取得している
- [ ] Cargo.tomlのバージョンを更新した
- [ ] `cargo build --release`が成功する
- [ ] `cargo test`が成功する
- [ ] リリースノートを作成した
- [ ] タグ名がCargo.tomlのバージョンと一致している
- [ ] プレリリースか正式版かを決定した

## 参考資料

- [セマンティックバージョニング](https://semver.org/lang/ja/)
- [GitHub CLI - リリース作成](https://cli.github.com/manual/gh_release_create)
- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)

## 更新履歴

- 2025-01-02: 初版作成
