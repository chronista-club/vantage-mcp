#!/bin/bash

# コミットメッセージからバージョンバンプ種別を判定
# 使用法: ./determine-bump-type.sh

set -e

# 前回のタグから現在までのコミットメッセージを取得
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
    # タグがない場合は全コミットを対象
    COMMITS=$(git log --pretty=format:"%s %b" --no-merges)
else
    # 前回のタグから現在までのコミット
    COMMITS=$(git log $LAST_TAG..HEAD --pretty=format:"%s %b" --no-merges)
fi

# デフォルトはパッチバンプ
BUMP_TYPE="patch"

# コミットメッセージを解析
if echo "$COMMITS" | grep -q "BREAKING CHANGE:\|breaking:"; then
    BUMP_TYPE="major"
elif echo "$COMMITS" | grep -q "^feat:\|^feat("; then
    BUMP_TYPE="minor"
elif echo "$COMMITS" | grep -q "^fix:\|^fix("; then
    BUMP_TYPE="patch"
fi

echo "$BUMP_TYPE"