# Ichimi Server Web UI 設計ドキュメント

## 概要

Ichimi ServerのWebダッシュボードは、プロセス管理のための直感的なユーザーインターフェースを提供します。Vue 3 + TypeScript + Viteをベースとした、モダンでレスポンシブなシングルページアプリケーション（SPA）です。

### 目的

- プロセスの視覚的な監視と管理
- リアルタイムでのプロセス状態の確認
- 簡単なプロセス操作（起動/停止/削除）
- クリップボード管理とテンプレート機能

## 技術スタック

### コアフレームワーク
- **Vue 3** (Composition API): リアクティブなUIフレームワーク
- **TypeScript**: 型安全性と開発体験の向上
- **Vite**: 高速な開発サーバーとビルドツール

### UI/デザイン
- **Tabler UI**: モダンなUIコンポーネントライブラリ
- **Tabler Icons**: アイコンセット

### 状態管理
- **Pinia**: Vue 3公式の状態管理ライブラリ

### ルーティング
- **Vue Router**: SPAのクライアントサイドルーティング

### ビルド/パッケージ管理
- **Bun**: 高速なJavaScriptランタイム＆パッケージマネージャー

## ディレクトリ構造

```
ui/web/
├── src/
│   ├── api/                # APIクライアント層
│   │   └── client.ts       # HTTP通信ロジック
│   ├── components/         # 再利用可能なコンポーネント
│   │   ├── layout/         # レイアウトコンポーネント
│   │   │   ├── AppHeader.vue
│   │   │   ├── NavigationBar.vue
│   │   │   └── StatsBar.vue
│   │   └── process/        # プロセス関連コンポーネント
│   │       ├── ProcessActions.vue   # アクションボタン
│   │       ├── ProcessCard.vue      # プロセスカード表示
│   │       ├── ProcessStatus.vue    # 状態バッジ
│   │       └── ProcessTable.vue     # テーブル表示
│   ├── router/             # ルーティング設定
│   │   └── index.ts
│   ├── stores/             # Pinia状態管理
│   │   ├── clipboard.ts    # クリップボード状態
│   │   ├── process.ts      # プロセス状態
│   │   ├── settings.ts     # アプリ設定
│   │   └── template.ts     # テンプレート状態
│   ├── types/              # TypeScript型定義
│   │   └── index.ts
│   ├── views/              # ページレベルコンポーネント
│   │   ├── ProcessesView.vue  # プロセス一覧（メインページ）
│   │   ├── TemplatesView.vue  # テンプレート管理
│   │   └── ClipboardView.vue  # クリップボード管理
│   ├── App.vue             # ルートコンポーネント
│   └── main.ts             # エントリーポイント
├── public/                 # 静的アセット
├── dist/                   # ビルド出力（gitignore）
├── index.html              # HTMLエントリーポイント
├── vite.config.ts          # Vite設定
└── package.json            # 依存関係
```

## アーキテクチャ

### コンポーネント階層

```
App.vue
├── AppHeader.vue
│   ├── NavigationBar.vue
│   └── StatsBar.vue (optional)
└── Router View
    ├── ProcessesView.vue
    │   ├── ProcessCard.vue (multiple)
    │   │   ├── ProcessStatus.vue
    │   │   └── ProcessActions.vue
    │   └── ProcessTable.vue (alternative view)
    ├── TemplatesView.vue
    └── ClipboardView.vue
```

### 状態管理（Pinia）

#### Process Store (`stores/process.ts`)
**責務**: プロセス一覧の管理、API呼び出し、自動更新

**State:**
- `processes`: プロセス一覧
- `loading`: ローディング状態
- `error`: エラーメッセージ

**Actions:**
- `loadProcesses()`: プロセス一覧を取得
- `startProcess(id)`: プロセス起動
- `stopProcess(id)`: プロセス停止
- `removeProcess(id)`: プロセス削除
- `startAutoRefresh()`: 自動更新開始
- `stopAutoRefresh()`: 自動更新停止

#### Settings Store (`stores/settings.ts`)
**責務**: アプリケーション設定の管理

**State:**
- `theme`: テーマ (light/dark)
- `auto_refresh`: 自動更新の有効/無効
- `refresh_interval`: 更新間隔（ミリ秒）
- `viewMode`: 表示モード (card/table)

**Actions:**
- `setTheme(theme)`: テーマ変更
- `setAutoRefresh(enabled)`: 自動更新設定
- `setRefreshInterval(ms)`: 更新間隔設定
- `setViewMode(mode)`: 表示モード切替

#### Clipboard Store (`stores/clipboard.ts`)
**責務**: クリップボード項目の管理

#### Template Store (`stores/template.ts`)
**責務**: プロセステンプレートの管理

### ルーティング

```typescript
/ → /processes (redirect)
/processes → ProcessesView (メインページ)
/templates → TemplatesView
/clipboard → ClipboardView
/* → /processes (404 redirect)
```

**設計方針:**
- ProcessesViewをデフォルトページに（v0.2.4で変更）
- シンプルな3ページ構成
- 404は常にProcessesにリダイレクト

## 主要コンポーネント

### ProcessesView.vue（メインページ）

**機能:**
- プロセス一覧表示
- 状態フィルタ（All/Running/Stopped/Failed）
- 統計サマリー表示（実行中/停止中/失敗）
- カード/テーブル表示切替
- 自動更新

**レイアウト:**
```
[ヘッダー]
  タイトル: Processes
  統計: [X Running] [Y Stopped] [Z Failed]

[コントロール]
  [フィルタ: All | Running | Stopped | Failed]
  [表示: Card | Table]
  [Refresh]

[コンテンツエリア]
  - カード表示: 縦1列でプロセスカード
  - テーブル表示: 表形式で一覧
```

### ProcessCard.vue

**表示内容:**
- プロセスID（太字）
- コマンドライン（コード表示）
- 状態バッジ
- 作業ディレクトリ
- 環境変数数
- アクションボタン（Start/Stop/Remove/Output）

### ProcessStatus.vue

**状態表示:**
- Running: 緑バッジ
- Stopped/NotStarted: グレーバッジ
- Failed: 赤バッジ

### ProcessActions.vue

**アクション:**
- Start: プロセス起動
- Stop: プロセス停止
- Remove: プロセス削除
- Show Output: ログ表示（TODO）

## データモデル（TypeScript型定義）

### ProcessState

Rustのenumに対応する複雑な型構造：

```typescript
type ProcessState =
  | 'NotStarted'
  | { Running: { pid: number; started_at: string } }
  | { Stopped: { exit_code?: number; stopped_at: string } }
  | { Failed: { error: string; failed_at: string } };
```

**ヘルパー関数:**
- `isRunning(state)`: Running状態かチェック
- `isStopped(state)`: Stopped状態かチェック
- `isFailed(state)`: Failed状態かチェック
- `isNotStarted(state)`: NotStarted状態かチェック
- `getStateLabel(state)`: 状態ラベル取得
- `getStateColor(state)`: 状態に応じた色取得

### ProcessInfo

```typescript
interface ProcessInfo {
  id: string;
  command: string;
  args: string[];
  cwd?: string;
  state: ProcessState;
  env?: Record<string, string>;
  auto_start_on_create?: boolean;
  auto_start_on_restore?: boolean;
}
```

## API統合

### API Client (`api/client.ts`)

**ベースURL:** `http://localhost:12700` (デフォルト)

**エンドポイント:**

#### プロセス管理
- `GET /api/processes` - プロセス一覧取得
- `POST /api/processes` - プロセス作成
- `POST /api/processes/:id/start` - プロセス起動
- `POST /api/processes/:id/stop` - プロセス停止
- `DELETE /api/processes/:id` - プロセス削除
- `GET /api/processes/:id/logs` - ログ取得

#### ダッシュボード
- `GET /api/dashboard` - ダッシュボードデータ取得

#### クリップボード
- `GET /api/clipboard` - クリップボード一覧
- `POST /api/clipboard` - クリップボード項目作成

## デザインシステム

### Tabler UI Framework

**カラーテーマ:**
- Primary: プライマリアクション（青系）
- Success: 成功状態（緑）
- Warning: 警告（黄）
- Danger: エラー・削除（赤）
- Secondary: 非アクティブ状態（グレー）

**コンポーネント:**
- `.card`: カード表示
- `.btn`: ボタン
- `.badge`: バッジ（状態表示）
- `.table`: テーブル
- `.empty`: 空状態表示

**レイアウト:**
- `.page-header`: ページヘッダー
- `.page-body`: メインコンテンツ
- `.container-xl`: コンテナ
- `.row-cards`: カードグリッド

## 開発ガイドライン

### コンポーネント設計原則

1. **Single Responsibility**: 1コンポーネント1責務
2. **Props Down, Events Up**: 親から子へprops、子から親へemit
3. **Composition API**: `<script setup>`を使用
4. **TypeScript**: 全て型定義を行う
5. **Pinia**: グローバル状態はストアで管理

### ファイル命名規則

- **コンポーネント**: PascalCase（`ProcessCard.vue`）
- **Store**: camelCase（`process.ts`）
- **型定義**: camelCase（`index.ts`）

### コードスタイル

```vue
<template>
  <!-- シンプルで読みやすいテンプレート -->
</template>

<script setup lang="ts">
// インポート
import { ref, computed } from 'vue';

// Props/Emits定義
interface Props {
  process: ProcessInfo;
}
const props = defineProps<Props>();

// Reactive状態
const isLoading = ref(false);

// Computed
const displayName = computed(() => props.process.id);

// Methods
async function handleAction() {
  // ...
}
</script>

<style scoped>
/* コンポーネント固有のスタイル */
</style>
```

### 状態管理のベストプラクティス

1. **ローカル状態**: コンポーネント内で完結する状態は`ref/reactive`
2. **共有状態**: 複数コンポーネントで使う状態はPiniaストア
3. **API呼び出し**: ストアのactionで行う
4. **エラーハンドリング**: try-catchで適切にハンドリング

## ビルドとデプロイ

### 開発モード

```bash
cd ui/web
bun install
bun run dev
```

開発サーバー: `http://localhost:5173`

### プロダクションビルド

```bash
bun run build
```

出力先: `ui/web/dist/`

**Rustサーバーからの配信:**
- `rust-embed`でビルド済みファイルを埋め込み
- サーバー起動時に`--web`オプションで有効化
- デフォルトポート: 12700

### 環境変数

```typescript
// vite.config.ts
export default defineConfig({
  base: import.meta.env.BASE_URL,
  // ...
});
```

## 今後の改善案

### 短期
- [ ] ログ表示モーダルの実装
- [ ] プロセス詳細ページ
- [ ] ダークモード切替UI
- [ ] リアルタイムログストリーミング

### 中期
- [ ] プロセスグループ管理
- [ ] 統計グラフ表示
- [ ] 通知機能
- [ ] キーボードショートカット

### 長期
- [ ] マルチサーバー対応
- [ ] パフォーマンスダッシュボード
- [ ] カスタムダッシュボード
- [ ] プラグインシステム

## トラブルシューティング

### よくある問題

**ビルドエラー: 型エラー**
- ProcessState型は複雑なので、ヘルパー関数を使用すること
- `.toLowerCase()`は使えない → `isRunning()`等を使う

**自動更新が動かない**
- Settings Storeで`auto_refresh`を確認
- コンポーネントのunmountedで`stopAutoRefresh()`を呼んでいるか確認

**APIエラー**
- サーバーが起動しているか確認（`cargo run --bin ichimi -- --web`）
- CORSエラーの場合、サーバー側の設定を確認

## 参考リンク

- [Vue 3 公式ドキュメント](https://vuejs.org/)
- [Pinia 公式ドキュメント](https://pinia.vuejs.org/)
- [Tabler UI](https://tabler.io/)
- [Vite 公式ドキュメント](https://vitejs.dev/)
- [TypeScript 公式ドキュメント](https://www.typescriptlang.org/)
