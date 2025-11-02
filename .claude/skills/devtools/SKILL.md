# Chrome DevTools スキル

Chrome DevTools MCPを使用したWebアプリケーションのデバッグとテストを支援するスキル。

## 概要

このスキルは、Claude CodeがChrome DevTools MCPサーバーを効果的に使用して、Webアプリケーションのデバッグ、テスト、品質保証を行うためのガイドラインとベストプラクティスを提供します。

## 主な機能

### 1. ページナビゲーションと管理
- ページの一覧表示、選択、作成、削除
- URL遷移とブラウザ履歴の操作
- 複数タブの管理

### 2. コンテンツの取得と操作
- アクセシビリティツリーによるスナップショット取得
- スクリーンショット（ページ全体/要素単位）
- 要素のクリック、ホバー、ドラッグ&ドロップ
- フォーム入力とファイルアップロード

### 3. デバッグとモニタリング
- コンソールメッセージの収集
- ネットワークリクエストの監視
- パフォーマンス分析（Core Web Vitals含む）
- ブラウザダイアログの処理

### 4. テストと検証
- JavaScript実行とDOM操作
- レスポンシブデザインのテスト
- ネットワーク/CPU throttlingのエミュレーション

## 使用開始前の確認

Chrome DevToolsを使用する前に、以下を確認してください：

1. **MCPサーバーの起動状態**
   ```bash
   # .mcp.jsonの設定を確認
   cat .mcp.json | grep chrome-devtools
   ```

2. **ページ一覧の取得**
   ```typescript
   mcp__chrome-devtools__list_pages()
   ```

3. **新しいページの作成（必要に応じて）**
   ```typescript
   mcp__chrome-devtools__new_page({ url: "http://localhost:5173" })
   ```

## 典型的なワークフロー

### WebUIのエラーチェック

1. **開発サーバーの起動**
   ```bash
   cd ui/web && bun run dev
   ```

2. **ページへのアクセス**
   ```typescript
   mcp__chrome-devtools__navigate_page({ url: "http://localhost:5173" })
   ```

3. **コンソールエラーの確認**
   ```typescript
   mcp__chrome-devtools__list_console_messages()
   ```

4. **ネットワークエラーの確認**
   ```typescript
   mcp__chrome-devtools__list_network_requests({ 
     resourceTypes: ["xhr", "fetch"] 
   })
   ```

5. **スナップショットで状態確認**
   ```typescript
   mcp__chrome-devtools__take_snapshot()
   ```

### UIテストの実行

1. **要素のクリック**
   ```typescript
   // スナップショットからuidを取得
   mcp__chrome-devtools__take_snapshot()
   // 要素をクリック
   mcp__chrome-devtools__click({ uid: "1_7" })
   ```

2. **フォーム入力**
   ```typescript
   mcp__chrome-devtools__fill({ uid: "2_10", value: "test input" })
   ```

3. **結果の検証**
   ```typescript
   mcp__chrome-devtools__wait_for({ text: "成功", timeout: 5000 })
   ```

## ベストプラクティス

### 1. 効率的なデバッグ

- **コンソールメッセージを最初に確認**：エラーの大部分はコンソールに表示される
- **ネットワークリクエストを確認**：APIエラーや遅延を特定
- **スナップショットで状態把握**：DOM構造とコンテンツを効率的に確認

### 2. テストの信頼性向上

- **wait_forを使用**：動的コンテンツの読み込み完了を待つ
- **タイムアウトを適切に設定**：遅い環境でも動作するように
- **エラーハンドリング**：予期しない状態に対処

### 3. パフォーマンス考慮

- **不要なスクリーンショットを避ける**：スナップショットで代替可能か検討
- **ページリロードを最小化**：状態を保持して効率的にテスト
- **並列実行の活用**：複数ページを同時にテスト可能

## 注意事項

### UID（要素ID）の取り扱い

- UIDはページ遷移やリロードで変わる
- 常に最新のスナップショットからUIDを取得する
- 要素のテキストやロールで特定することを推奨

### タイムアウトとエラー

- デフォルトタイムアウトは30秒
- ネットワークが遅い場合は適宜調整
- エラー時は詳細なメッセージが返される

### ブラウザの状態管理

- ページは自動的にクローズされない
- 不要なページは手動でクローズ推奨
- 最後のページはクローズ不可

## 参考資料

詳細なツール使用方法とパターン集は以下を参照：

- [tools_reference.md](./reference/tools_reference.md) - 全ツールの詳細リファレンス
- [debugging_patterns.md](./patterns/debugging_patterns.md) - デバッグパターン集
- [testing_patterns.md](./patterns/testing_patterns.md) - テストパターン集
- [common_issues.md](./patterns/common_issues.md) - よくある問題と解決方法

## 更新履歴

- 2025-01-01: 初版作成
