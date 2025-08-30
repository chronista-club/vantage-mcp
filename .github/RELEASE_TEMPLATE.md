# Release Template for Ichimi Server

以下のテンプレートを使用してGitHubリリースを作成してください。

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

## コマンド例

```bash
# タグを作成
git tag -a vX.X.X-betaXX -m "Release vX.X.X-betaXX - 簡潔な説明"

# タグをプッシュ
git push origin vX.X.X-betaXX

# GitHubリリースを作成
gh release create vX.X.X-betaXX \
  --title "vX.X.X-betaXX - タイトル" \
  --notes "$(cat release-notes.md)" \
  --prerelease
```