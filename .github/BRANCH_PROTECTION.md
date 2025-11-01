# ブランチ保護ルール

## mainブランチの保護設定

このリポジトリのmainブランチには以下の保護ルールが適用されています：

### プルリクエストレビュー要件
- ✅ **レビュー承認が必須**: mainブランチへのマージには最低1人のレビュアーからの承認が必要です
- ✅ **古いレビューの無効化**: 新しいコミットがプッシュされた場合、以前の承認は無効になります
- ✅ **会話の解決が必須**: すべてのコメントが解決されるまでマージできません

### 制限事項
- ❌ **直接プッシュ禁止**: mainブランチへの直接プッシュはできません
- ❌ **強制プッシュ禁止**: force pushは許可されていません
- ❌ **ブランチ削除禁止**: mainブランチは削除できません

### 管理者設定
- ⚠️ **管理者も制限対象**: 管理者もこれらのルールに従う必要があります

## 開発フロー

1. **フィーチャーブランチの作成**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **変更の実装とコミット**
   ```bash
   git add .
   git commit -m "feat: 新機能の実装"
   ```

3. **プルリクエストの作成**
   ```bash
   gh pr create --title "新機能の追加" --body "詳細な説明"
   ```

4. **レビューの依頼**
   - プルリクエストを作成後、レビュアーをアサインしてください
   - @makoto.ito からの承認が必要です

5. **マージ**
   - レビュー承認後、PRをマージできます
   - マージ方法: Squash and merge を推奨

## ブランチ保護ルールの更新

ブランチ保護ルールを更新する場合は、`branch-protection.json`を編集して以下のコマンドを実行：

```bash
gh api repos/chronista-club/vantage-mcp/branches/main/protection \
  -X PUT \
  --input branch-protection.json
```

## トラブルシューティング

### 緊急時のバイパス

緊急の修正が必要な場合でも、以下の手順を守ってください：

1. 緊急修正用のブランチを作成
2. 最小限の変更を実装
3. プルリクエストを作成し、タイトルに「[URGENT]」を付ける
4. レビュアーに緊急対応を依頼

### 保護ルールの一時的な無効化

管理者権限を持つユーザーのみが実行可能：

```bash
# 保護ルールの無効化（非推奨）
gh api repos/chronista-club/vantage-mcp/branches/main/protection -X DELETE

# 保護ルールの再有効化
gh api repos/chronista-club/vantage-mcp/branches/main/protection \
  -X PUT \
  --input branch-protection.json
```

⚠️ **注意**: 保護ルールの無効化は最後の手段としてのみ使用してください。