# テンプレート管理ページ - 仕様書

## 目的

頻繁に使用するプロセス設定をテンプレートとして保存し、**再利用性を高める**ことで、プロセス作成の手間を削減するページ。

### 解決する課題

- **設定の重複入力**: 同じような設定を何度も入力するのは非効率
- **設定ミス**: 手動入力によるタイプミスや設定漏れ
- **知識の共有**: 便利な設定を他の開発者と共有しにくい

## ユースケース

### UC-001: よく使うプロセスの登録

**Actor**: 開発者

**Goal**: Next.js 開発サーバーの設定をテンプレートとして保存したい

**Flow**:
1. テンプレート管理ページを開く
2. 「新規テンプレート作成」ボタンをクリック
3. テンプレート情報を入力:
   - 名前: "Next.js Dev Server"
   - 説明: "Next.js 開発サーバー（ポート 3000）"
   - コマンド: "npm"
   - 引数: ["run", "dev"]
   - 環境変数: { "PORT": "3000", "NODE_ENV": "development" }
4. 「保存」ボタンをクリック

**Success Criteria**: テンプレートが一覧に表示され、次回から 1 クリックでプロセス作成可能

### UC-002: テンプレートからプロセス作成

**Actor**: 開発者

**Goal**: 保存したテンプレートから新しいプロセスを作成したい

**Flow**:
1. テンプレート一覧で "Next.js Dev Server" を選択
2. 「使用する」ボタンをクリック
3. モーダルが開き、プロセス ID を入力: "frontend-dev"
4. 必要に応じてパラメータをカスタマイズ（ポート変更など）
5. 「作成」ボタンをクリック

**Success Criteria**: プロセスが作成され、プロセス管理ページに追加される

### UC-003: テンプレートの編集

**Actor**: 開発者

**Goal**: 既存テンプレートの設定を更新したい

**Flow**:
1. テンプレート一覧で "Next.js Dev Server" の「編集」ボタンをクリック
2. 環境変数を追加: { "ANALYZE": "true" }
3. 「保存」ボタンをクリック

**Success Criteria**: テンプレートが更新され、次回使用時に新しい設定が反映される

### UC-004: 使用頻度の追跡

**Actor**: 開発者

**Goal**: よく使うテンプレートを上位に表示したい

**Flow**:
1. テンプレートを使用するたびに use_count がインクリメントされる
2. テンプレート一覧がデフォルトで use_count 降順にソートされる

**Success Criteria**: 最も使用頻度が高いテンプレートが一覧の上部に表示される

## 機能要件

### FR-001: テンプレート一覧表示（必須）

- **説明**: 登録済みテンプレートをリスト表示
- **表示項目**:
  - テンプレート名
  - 説明
  - 使用回数
  - 作成日時
  - 最終使用日時
- **ソート**: 使用回数、作成日時、名前でソート可能
- **検索**: テンプレート名・説明で全文検索

### FR-002: テンプレート作成（必須）

- **説明**: 新しいテンプレートを登録
- **入力項目**:
  - テンプレート名（必須、一意）
  - 説明（オプション）
  - カテゴリ（オプション: Web, CLI, Database, Other）
  - コマンド（必須）
  - 引数（配列）
  - 環境変数（キー・バリュー）
  - 作業ディレクトリ
  - 自動起動フラグ（デフォルト: false）
- **バリデーション**:
  - テンプレート名は 1〜50 文字
  - 既存の名前と重複不可

### FR-003: テンプレート編集（必須）

- **説明**: 既存テンプレートの設定を更新
- **制約**: 使用回数はリセットされない

### FR-004: テンプレート削除（必須）

- **説明**: テンプレートを削除
- **確認ダイアログ**: 削除前に確認を求める
- **注意**: 削除してもテンプレートから作成されたプロセスには影響しない

### FR-005: テンプレート使用（必須）

- **説明**: テンプレートからプロセスを作成
- **Flow**:
  1. テンプレート選択
  2. プロセス ID 入力（デフォルト: `template-name-{timestamp}`）
  3. パラメータカスタマイズ（オプション）
  4. プロセス作成 API 呼び出し
  5. use_count インクリメント
  6. last_used_at 更新

### FR-006: テンプレートのエクスポート/インポート（オプション）

- **説明**: テンプレートを JSON ファイルとしてエクスポート/インポート
- **用途**: チーム間でテンプレート共有
- **フォーマット**:
```json
{
  "templates": [
    {
      "name": "Next.js Dev Server",
      "description": "Next.js development server",
      "command": "npm",
      "args": ["run", "dev"],
      "env": { "PORT": "3000" }
    }
  ]
}
```

### FR-007: カテゴリフィルター（オプション）

- **説明**: カテゴリでテンプレートをフィルタリング
- **カテゴリ**: Web, CLI, Database, Other

## 非機能要件

### NFR-001: パフォーマンス

- **初期ロード**: 50 テンプレートの一覧表示を 500ms 以内
- **検索**: 100 テンプレートから検索結果を 100ms 以内に表示

### NFR-002: データ永続化

- **保存先**: SurrealDB（`template` テーブル）
- **バックアップ**: プロセスと同様に YAML スナップショットに含める

### NFR-003: 多言語対応

- **サポート言語**: 日本語、英語
- **UI 文言**: i18n で管理

## 設計

### UI コンポーネント構成

```
TemplatesView.vue
├── TemplateFilters.vue (検索・カテゴリフィルター)
├── TemplateList.vue
│   └── TemplateCard.vue × N
│       ├── TemplateInfo.vue
│       └── TemplateActions.vue
├── CreateTemplateModal.vue
├── EditTemplateModal.vue
└── UseTemplateModal.vue
```

### データモデル（SurrealDB）

```surql
DEFINE TABLE template SCHEMAFULL;

DEFINE FIELD name ON template TYPE string
  ASSERT $value != NONE AND string::len($value) > 0 AND string::len($value) <= 50;

DEFINE FIELD description ON template TYPE option<string>;

DEFINE FIELD category ON template TYPE option<string>
  ASSERT $value == NONE OR $value IN ["Web", "CLI", "Database", "Other"];

DEFINE FIELD command ON template TYPE string
  ASSERT $value != NONE AND string::len($value) > 0;

DEFINE FIELD args ON template TYPE array
  ASSERT $value == NONE OR type::is::array($value);

DEFINE FIELD env ON template TYPE object
  DEFAULT {};

DEFINE FIELD cwd ON template TYPE option<string>;

DEFINE FIELD auto_start_on_restore ON template TYPE bool
  DEFAULT false;

DEFINE FIELD use_count ON template TYPE int
  DEFAULT 0;

DEFINE FIELD created_at ON template TYPE string;

DEFINE FIELD last_used_at ON template TYPE option<string>;

DEFINE INDEX unique_template_name ON template FIELDS name UNIQUE;
```

### 状態管理（Pinia）

```typescript
// stores/template.ts
interface TemplateState {
  templates: Map<string, Template>;
  filters: {
    category: string | null;
    searchQuery: string;
  };
  sortBy: 'use_count' | 'created_at' | 'name';
  sortOrder: 'asc' | 'desc';
}

actions:
- fetchTemplates()
- createTemplate(template)
- updateTemplate(name, updates)
- deleteTemplate(name)
- useTemplate(name, processId, customizations)
- incrementUseCount(name)
- exportTemplates()
- importTemplates(json)
- setFilter(key, value)
- setSorting(sortBy, sortOrder)
```

### API エンドポイント

```
GET  /api/templates              - テンプレート一覧取得
POST /api/templates              - テンプレート作成
GET  /api/templates/:name        - テンプレート詳細取得
PUT  /api/templates/:name        - テンプレート更新
DELETE /api/templates/:name      - テンプレート削除
POST /api/templates/:name/use    - テンプレート使用（プロセス作成）
POST /api/templates/import       - テンプレートインポート
GET  /api/templates/export       - テンプレートエクスポート
```

## テストケース

### TC-001: テンプレート作成

**前提**: テンプレート管理ページを開いている

**操作**:
1. 「新規作成」ボタンをクリック
2. フォームに入力:
   - 名前: "PostgreSQL"
   - コマンド: "docker"
   - 引数: ["run", "-p", "5432:5432", "postgres"]
3. 「保存」ボタンをクリック

**期待結果**:
- API `POST /api/templates` が呼ばれる
- テンプレートが一覧に追加される
- use_count = 0
- created_at が現在時刻

### TC-002: テンプレート使用

**前提**: テンプレート "PostgreSQL" が存在

**操作**:
1. 「使用する」ボタンをクリック
2. プロセス ID を入力: "db-dev"
3. 「作成」ボタンをクリック

**期待結果**:
- API `POST /api/templates/PostgreSQL/use` が呼ばれる
- プロセス "db-dev" が作成される
- use_count が 1 にインクリメント
- last_used_at が更新される

### TC-003: テンプレート削除（確認）

**前提**: テンプレート "Old Template" が存在

**操作**:
1. 「削除」ボタンをクリック
2. 確認ダイアログで "OK"

**期待結果**:
- 確認ダイアログが表示される
- API `DELETE /api/templates/Old Template` が呼ばれる
- テンプレートが一覧から消える

### TC-004: バリデーション（名前重複）

**前提**: テンプレート "Next.js" が存在

**操作**:
1. 新規作成で名前を "Next.js" に設定
2. 「保存」ボタンをクリック

**期待結果**:
- API がエラーを返す
- トースト通知で「テンプレート名が重複しています」を表示
- フォームが閉じない

### TC-005: 検索

**前提**: 10 テンプレート（3 つが "Next" を含む）

**操作**: 検索ボックスに "Next" を入力

**期待結果**:
- 3 つのテンプレートのみが表示される
- リアルタイムで絞り込まれる

### TC-006: ソート

**前提**: 5 テンプレート（use_count: 10, 5, 3, 1, 0）

**操作**: ソート条件を「使用回数（降順）」に設定

**期待結果**:
- use_count = 10 のテンプレートが最上位
- use_count = 0 のテンプレートが最下位

## 制約

### 技術的制約

- **テンプレート数上限**: 100 個まで（パフォーマンス考慮）
- **名前の長さ**: 1〜50 文字

### ビジネス制約

- **共有機能**: 現時点ではローカル環境のみ（将来的にクラウド同期を検討）

## 今後の拡張

- **テンプレートのバージョン管理**: 変更履歴を保存し、以前のバージョンに戻せる
- **公開テンプレートギャラリー**: コミュニティでテンプレートを共有
- **テンプレート変数**: `{{PORT}}` のような変数を定義し、使用時に値を入力
- **依存関係の定義**: 「このテンプレートを使う前に DB を起動」のような依存関係

## 更新履歴

| 日付 | 変更者 | 変更内容 | 理由 |
|------|--------|----------|------|
| 2025-11-02 | Claude Code | 初版作成 | 仕様の明確化 |
