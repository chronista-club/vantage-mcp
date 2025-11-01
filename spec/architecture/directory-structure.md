# ディレクトリ構成仕様 - 仕様書

## 目的

Vantage MCP プロジェクト全体のディレクトリ構成を明確に定義し、**保守性・拡張性・可読性**を向上させる。

### 解決する課題

- **ファイル配置の曖昧さ**: どこに何を置くべきか不明確
- **命名の不統一**: ディレクトリ名やファイル名の規則が統一されていない
- **スケールの問題**: プロジェクトが大きくなるにつれて整理が困難

## 設計原則

### 1. **機能ベースの分割**

関連する機能を同じディレクトリにまとめる。

**良い例**:
```
src/features/process-management/
  ├── components/
  ├── stores/
  └── api/
```

**悪い例**:
```
src/components/  # 全てのコンポーネントが混在
src/stores/      # 全てのストアが混在
```

### 2. **責務の明確化**

各ディレクトリが単一の責務を持つようにする。

### 3. **フラットな構造**

不必要にネストを深くしない（最大 3〜4 階層）。

### 4. **一貫性のある命名**

- ディレクトリ名: `kebab-case`（例: `process-management`）
- ファイル名: `PascalCase.vue`（コンポーネント）、`kebab-case.ts`（その他）

## プロジェクト全体の構成

```
vantage-mcp-ui/
├── .claude/                    # Claude Code 設定
│   ├── skills/                 # スキルファイル
│   │   ├── webxr-designer/
│   │   └── mcp-builder/
│   └── CLAUDE.md               # プロジェクト指示
│
├── .github/                    # GitHub Actions CI/CD
│   ├── workflows/
│   └── BRANCH_PROTECTION.md
│
├── crates/                     # Rust ワークスペース
│   ├── vantage-atom/           # メインサーバークレート
│   │   ├── src/
│   │   │   ├── lib.rs          # MCP ツールハンドラー
│   │   │   ├── messages/       # リクエスト/レスポンス型
│   │   │   ├── process/        # プロセス管理ロジック
│   │   │   ├── ci/             # CI 監視
│   │   │   └── web/            # Web サーバー
│   │   ├── tests/
│   │   └── Cargo.toml
│   │
│   ├── vantage-atom-persistence/  # 永続化レイヤー
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── persistence/
│   │   └── Cargo.toml
│   │
│   └── vantage-mcp/            # CLI バイナリクレート
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── ui/                         # フロントエンド
│   └── web/                    # Vue 3 WebUI
│       ├── public/             # 静的ファイル
│       ├── src/
│       │   ├── features/       # 機能モジュール（提案）
│       │   │   ├── processes/
│       │   │   ├── templates/
│       │   │   ├── clipboard/
│       │   │   └── settings/
│       │   ├── shared/         # 共通モジュール（提案）
│       │   │   ├── components/
│       │   │   ├── composables/
│       │   │   ├── lib/
│       │   │   ├── styles/
│       │   │   └── types/
│       │   ├── core/           # コアシステム（提案）
│       │   │   ├── api/
│       │   │   ├── i18n/
│       │   │   ├── router/
│       │   │   └── theme/
│       │   ├── App.vue
│       │   └── main.ts
│       ├── index.html
│       ├── package.json
│       ├── tsconfig.json
│       └── vite.config.ts
│
├── spec/                       # 仕様書（このドキュメント）
│   ├── README.md
│   ├── pages/
│   ├── processes/
│   └── architecture/
│
├── docs/                       # ドキュメント
├── tests/                      # 統合テスト
├── scripts/                    # ビルド・デプロイスクリプト
├── Cargo.toml                  # Rust ワークスペース設定
├── README.md
└── CHANGELOG.md
```

## ui/web の推奨構成（リファクタリング案）

### 現状の課題

1. **features/ がない**: プロセス、テンプレート、クリップボードが分散
2. **shared/ がない**: 共通コンポーネントの配置が曖昧
3. **core/ がない**: API、i18n、router が src 直下に混在

### 提案する構成

```
ui/web/src/
├── features/                   # 機能モジュール（ドメイン駆動）
│   ├── processes/
│   │   ├── components/
│   │   │   ├── ProcessCard.vue
│   │   │   ├── ProcessTable.vue
│   │   │   ├── Process3DGraph.vue
│   │   │   ├── ProcessStatus.vue
│   │   │   └── ProcessActions.vue
│   │   ├── composables/
│   │   │   └── useProcessFilters.ts
│   │   ├── stores/
│   │   │   └── process.ts
│   │   ├── views/
│   │   │   └── ProcessesView.vue
│   │   └── types.ts
│   │
│   ├── templates/
│   │   ├── components/
│   │   │   ├── TemplateCard.vue
│   │   │   ├── CreateTemplateModal.vue
│   │   │   └── UseTemplateModal.vue
│   │   ├── stores/
│   │   │   └── template.ts
│   │   ├── views/
│   │   │   └── TemplatesView.vue
│   │   └── types.ts
│   │
│   ├── clipboard/
│   │   ├── components/
│   │   ├── stores/
│   │   │   └── clipboard.ts
│   │   ├── views/
│   │   │   └── ClipboardView.vue
│   │   └── types.ts
│   │
│   └── settings/
│       ├── components/
│       │   └── SettingsDropdown.vue
│       ├── stores/
│       │   └── settings.ts
│       ├── views/
│       │   └── SettingsView.vue
│       └── types.ts
│
├── shared/                     # 共通モジュール（再利用可能）
│   ├── components/
│   │   ├── layout/
│   │   │   └── AppHeader.vue
│   │   ├── ui/                 # 汎用 UI コンポーネント
│   │   │   ├── Button.vue
│   │   │   ├── Modal.vue
│   │   │   └── Toast.vue
│   │   └── 3d/                 # 3D 関連コンポーネント
│   │       └── Scene3D.vue
│   │
│   ├── composables/
│   │   ├── useToast.ts
│   │   └── useTheme.ts
│   │
│   ├── lib/
│   │   ├── 3d/
│   │   │   └── LayoutManager.ts
│   │   └── utils/
│   │       └── formatDate.ts
│   │
│   ├── styles/
│   │   ├── main.scss
│   │   └── variables.scss
│   │
│   └── types/
│       └── index.ts
│
├── core/                       # コアシステム（アプリ基盤）
│   ├── api/
│   │   └── client.ts
│   │
│   ├── i18n/
│   │   ├── index.ts
│   │   └── locales/
│   │       ├── ja.json
│   │       └── en.json
│   │
│   ├── router/
│   │   └── index.ts
│   │
│   └── theme/
│       ├── themes.ts
│       └── colors.ts
│
├── App.vue
└── main.ts
```

### モジュール間の依存関係ルール

```
features/*  →  shared/*  ✅ OK
features/*  →  core/*    ✅ OK
features/*  ↔  features/* ❌ NG（他の feature に依存しない）

shared/*    →  core/*    ✅ OK
shared/*    ↔  features/* ❌ NG（feature に依存しない）

core/*      ↔  features/* ❌ NG
core/*      ↔  shared/*   ❌ NG
```

### リファクタリング手順

1. **Phase 1: ディレクトリ作成**
   ```bash
   mkdir -p src/{features,shared,core}
   mkdir -p src/features/{processes,templates,clipboard,settings}
   mkdir -p src/shared/{components,composables,lib,styles,types}
   mkdir -p src/core/{api,i18n,router,theme}
   ```

2. **Phase 2: features/ への移動**
   - `src/views/ProcessesView.vue` → `src/features/processes/views/ProcessesView.vue`
   - `src/components/process/*.vue` → `src/features/processes/components/*.vue`
   - `src/stores/process.ts` → `src/features/processes/stores/process.ts`

3. **Phase 3: shared/ への移動**
   - `src/components/layout/*.vue` → `src/shared/components/layout/*.vue`
   - `src/composables/*.ts` → `src/shared/composables/*.ts`
   - `src/lib/3d/*.ts` → `src/shared/lib/3d/*.ts`
   - `src/styles/*.scss` → `src/shared/styles/*.scss`
   - `src/types/*.ts` → `src/shared/types/*.ts`

4. **Phase 4: core/ への移動**
   - `src/api/*.ts` → `src/core/api/*.ts`
   - `src/i18n/**` → `src/core/i18n/**`
   - `src/router/*.ts` → `src/core/router/*.ts`
   - `src/themes.ts` → `src/core/theme/themes.ts`

5. **Phase 5: インポートパスの更新**
   - 全てのインポートパスを新しい構造に合わせて修正
   - エイリアスの活用（`@/` → `src/`）

6. **Phase 6: 古いディレクトリの削除**
   ```bash
   rm -rf src/components
   rm -rf src/views
   rm -rf src/stores
   rm -rf src/api
   rm -rf src/i18n
   rm -rf src/router
   rm src/themes.ts
   ```

## ファイル命名規則

### コンポーネント（.vue）

- **PascalCase**: `ProcessCard.vue`, `AppHeader.vue`
- **説明的な名前**: `CreateTemplateModal.vue`（動詞 + 名詞 + 種類）

### TypeScript（.ts）

- **kebab-case**: `use-toast.ts`, `format-date.ts`
- **Composables**: `use-` プレフィックス
- **Store**: 機能名そのまま `process.ts`, `template.ts`

### スタイルファイル（.scss）

- **kebab-case**: `main.scss`, `variables.scss`

### テストファイル

- **同じ名前 + .test**: `ProcessCard.test.ts`, `use-toast.test.ts`

## インポートエイリアス

`tsconfig.json` でエイリアスを設定：

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@features/*": ["src/features/*"],
      "@shared/*": ["src/shared/*"],
      "@core/*": ["src/core/*"]
    }
  }
}
```

**使用例**:
```typescript
// Bad
import { ProcessCard } from '../../../features/processes/components/ProcessCard.vue';

// Good
import { ProcessCard } from '@features/processes/components/ProcessCard.vue';
import { useToast } from '@shared/composables/use-toast';
import { apiClient } from '@core/api/client';
```

## テストケース

### TC-001: features/ のモジュール独立性

**検証**: features/processes が features/templates に依存していないか

**方法**: インポートパスを grep で検索

```bash
grep -r "from '@features/templates" src/features/processes/
# 結果: 何も見つからない → OK
```

### TC-002: shared/ の汎用性

**検証**: shared/ が features/ に依存していないか

**方法**:
```bash
grep -r "from '@features" src/shared/
# 結果: 何も見つからない → OK
```

### TC-003: ビルドの成功

**検証**: リファクタリング後もビルドが成功するか

**方法**:
```bash
cd ui/web
bun run build
# 結果: エラーなし → OK
```

## 今後の拡張

### モジュールのバンドル分割

各 feature を独立したチャンクとしてビルド：

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'feature-processes': ['./src/features/processes'],
          'feature-templates': ['./src/features/templates'],
          'shared': ['./src/shared'],
        },
      },
    },
  },
});
```

### モノレポへの移行（将来）

複数の UI を管理する場合：

```
packages/
├── web-ui/         # 現在の Vue 3 UI
├── mobile-ui/      # React Native（将来）
└── shared/         # 共通ライブラリ
```

## 更新履歴

| 日付 | 変更者 | 変更内容 | 理由 |
|------|--------|----------|------|
| 2025-11-02 | Claude Code | 初版作成 | 仕様の明確化 |
