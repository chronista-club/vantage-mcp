#!/bin/bash
# リリースノート生成スクリプト
# 使用方法: ./generate-release-notes.sh <previous-tag> <new-version>
# 例: ./generate-release-notes.sh v0.1.0-beta20 0.1.0-beta21

set -e

PREV_TAG=$1
NEW_VERSION=$2

if [ -z "$PREV_TAG" ] || [ -z "$NEW_VERSION" ]; then
    echo "使用方法: ./generate-release-notes.sh <previous-tag> <new-version>"
    echo "例: ./generate-release-notes.sh v0.1.0-beta20 0.1.0-beta21"
    exit 1
fi

NEW_TAG="v${NEW_VERSION}"
OUTPUT_FILE="release-notes.md"

echo "📝 リリースノートを生成します..."
echo "  前回タグ: ${PREV_TAG}"
echo "  新規タグ: ${NEW_TAG}"
echo ""

# リリースノートのヘッダー
cat > "${OUTPUT_FILE}" << EOF
# ${NEW_TAG}

リリース日: $(date '+%Y年%m月%d日')

## 📦 インストール方法

\`\`\`bash
cargo install --git https://github.com/chronista-club/vantage-mcp --tag ${NEW_TAG} vantage-mcp
\`\`\`

## 📋 変更内容

EOF

# コミット履歴から変更内容を抽出
echo "## 🎉 新機能" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline --grep="^feat" | sed 's/^[^ ]* /- /' >> "${OUTPUT_FILE}" || echo "- なし" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

echo "## 🐛 バグ修正" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline --grep="^fix" | sed 's/^[^ ]* /- /' >> "${OUTPUT_FILE}" || echo "- なし" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

echo "## 🔧 改善・リファクタリング" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline --grep="^refactor\|^perf" | sed 's/^[^ ]* /- /' >> "${OUTPUT_FILE}" || echo "- なし" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

echo "## 📚 ドキュメント" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline --grep="^docs" | sed 's/^[^ ]* /- /' >> "${OUTPUT_FILE}" || echo "- なし" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

echo "## 🎨 スタイル・UI改善" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline --grep="^style" | sed 's/^[^ ]* /- /' >> "${OUTPUT_FILE}" || echo "- なし" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

# 全コミットリスト
echo "## 📝 全コミット" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --oneline | sed 's/^/- /' >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

# 貢献者リスト
echo "## 👥 貢献者" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"
git log ${PREV_TAG}..HEAD --format='%aN' | sort -u | sed 's/^/- @/' >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"

echo "---" >> "${OUTPUT_FILE}"
echo "" >> "${OUTPUT_FILE}"
echo "**完全な変更履歴**: https://github.com/chronista-club/vantage-mcp/compare/${PREV_TAG}...${NEW_TAG}" >> "${OUTPUT_FILE}"

echo "✅ リリースノートを生成しました: ${OUTPUT_FILE}"
echo ""
echo "📄 内容を確認・編集してください:"
echo "  cat ${OUTPUT_FILE}"
echo ""
echo "✏️  編集が必要な場合:"
echo "  vim ${OUTPUT_FILE}"
