# Release Template for Ichimi Server

以下のテンプレートを使用してGitHubリリースを作成してください。

## リリース前チェックリスト

- [ ] Cargo.tomlのバージョンを更新した
- [ ] cargo build --release が成功する
- [ ] cargo test が全て成功する
- [ ] Cargo.lockが更新されている

## リリースノートテンプレート

```markdown
## 🎉 Ichimi Server vX.X.X-betaXX

### 📦 インストール方法

```bash
# cargo installを使用（推奨）
cargo install --git https://github.com/chronista-club/ichimi-server --tag vX.X.X-betaXX

# または最新版をインストール
cargo install --git https://github.com/chronista-club/ichimi-server

# ソースからビルド
git clone https://github.com/chronista-club/ichimi-server.git
cd ichimi-server
git checkout vX.X.X-betaXX
cargo build --release
```

### 🔧 主な変更内容

#### 機能追加
- 

#### バグ修正
- 

#### 改善
- 

### 📊 テスト結果
- ✅ XX個のテストケース全て成功

### 🔄 前回からの変更点 (vX.X.X-betaXX以降)
- 

### 📝 設定例

`.mcp.json`:
```json
{
  "mcpServers": {
    "ichimi": {
      "command": "ichimi",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### 🙏 謝辞
このリリースはClaude Codeを使用して開発されました。

---

**Full Changelog**: https://github.com/chronista-club/ichimi-server/compare/vX.X.X-betaXX...vX.X.X-betaXX
```

## リリース作成手順

```bash
# 1. Cargo.tomlのバージョンを更新
# 例: version = "0.1.0-beta11" → version = "0.1.0-beta12"
vim Cargo.toml

# 2. ビルドとテスト
cargo build --release
cargo test

# 3. バージョン更新をコミット
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to vX.X.X-betaXX"

# 4. タグを作成
git tag -a vX.X.X-betaXX -m "Release vX.X.X-betaXX - 簡潔な説明"

# 5. プッシュ
git push origin main
git push origin vX.X.X-betaXX

# 6. GitHubリリースを作成
gh release create vX.X.X-betaXX \
  --title "vX.X.X-betaXX - タイトル" \
  --notes "$(cat release-notes.md)" \
  --prerelease
```