#!/bin/bash
# Vantage MCP リリーススクリプト
# 使用方法: ./release.sh <version>
# 例: ./release.sh 0.1.0-beta21

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "エラー: バージョンを指定してください"
    echo "使用方法: ./release.sh <version>"
    echo "例: ./release.sh 0.1.0-beta21"
    exit 1
fi

TAG="v${VERSION}"

echo "🚀 リリースプロセスを開始します: ${TAG}"
echo ""

# ステップ1: 事前確認
echo "📋 ステップ1: 事前確認"
echo "  - ブランチを確認..."
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "    ❌ エラー: mainブランチにいません（現在: $BRANCH）"
    exit 1
fi
echo "    ✅ mainブランチにいます"

echo "  - リモートと同期状態を確認..."
git fetch origin
LOCAL=$(git rev-parse @)
REMOTE=$(git rev-parse @{u})
if [ "$LOCAL" != "$REMOTE" ]; then
    echo "    ❌ エラー: リモートと同期していません"
    exit 1
fi
echo "    ✅ リモートと同期しています"

echo "  - 未コミットの変更を確認..."
if ! git diff-index --quiet HEAD --; then
    echo "    ❌ エラー: 未コミットの変更があります"
    git status --short
    exit 1
fi
echo "    ✅ 未コミットの変更はありません"
echo ""

# ステップ2: Cargo.tomlのバージョン更新
echo "📝 ステップ2: Cargo.tomlのバージョン更新"
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "  現在のバージョン: ${CURRENT_VERSION}"
echo "  新しいバージョン: ${VERSION}"

# バージョンを更新
sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
rm Cargo.toml.bak
echo "    ✅ Cargo.tomlを更新しました"
echo ""

# ステップ3: Cargo.lockを更新
echo "🔧 ステップ3: Cargo.lockを更新"
cargo build > /dev/null 2>&1
echo "    ✅ Cargo.lockを更新しました"
echo ""

# ステップ4: ビルドとテスト
echo "🏗️  ステップ4: ビルドとテスト"
echo "  - リリースビルド..."
if ! cargo build --release > /dev/null 2>&1; then
    echo "    ❌ エラー: リリースビルドに失敗しました"
    git checkout Cargo.toml Cargo.lock
    exit 1
fi
echo "    ✅ リリースビルド成功"

echo "  - テスト実行..."
if ! cargo test > /dev/null 2>&1; then
    echo "    ❌ エラー: テストに失敗しました"
    git checkout Cargo.toml Cargo.lock
    exit 1
fi
echo "    ✅ テスト成功"
echo ""

# ステップ5: コミットとタグ作成
echo "📦 ステップ5: コミットとタグ作成"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${TAG}"
echo "    ✅ コミット作成"

git tag -a "${TAG}" -m "Release ${TAG}"
echo "    ✅ タグ作成"
echo ""

# ステップ6: プッシュ
echo "🚢 ステップ6: リモートにプッシュ"
read -p "リモートにプッシュしますか？ (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push origin main
    git push origin "${TAG}"
    echo "    ✅ プッシュ完了"
else
    echo "    ⏸️  プッシュをスキップしました"
    echo "    手動でプッシュする場合:"
    echo "      git push origin main"
    echo "      git push origin ${TAG}"
fi
echo ""

# ステップ7: GitHubリリース作成
echo "🎉 ステップ7: GitHubリリース作成"
echo "以下のコマンドでGitHubリリースを作成してください:"
echo ""
echo "gh release create ${TAG} \\"
echo "  --title \"${TAG} - タイトル\" \\"
echo "  --notes-file release-notes.md \\"
echo "  --prerelease"
echo ""

echo "✅ リリースプロセスが完了しました！"
