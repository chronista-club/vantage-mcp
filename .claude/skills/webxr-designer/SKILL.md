# WebXR Designer Skill

## 概要

WebXR Designer は、Three.js ベースの本格的な 3D UI フレームワークと WebXR (VR/AR) を統合したアプリケーション開発を支援するスキルです。

## 主な機能

### 1. 3D UI フレームワーク設計支援
- **マルチモード 3D レイアウトシステム**
  - List Mode: 垂直スクロール可能な 3D カードリスト
  - Graph Mode: 3D ネットワークグラフ表示
  - Control Mode: インタラクティブな 3D コントロールパネル
  - Edit Mode: 拡大された 3D 編集インターフェース

- **シームレスなトランジション**
  - TWEEN.js による滑らかなモード間遷移
  - 物理演算ベースのアニメーション
  - カメラワークの自動最適化

### 2. WebXR 統合
- **VR/AR デバイス対応**
  - WebXR Device API の統合
  - ハンドコントローラー入力
  - 視線追跡とジェスチャー認識

- **没入型 UI パターン**
  - 空間 UI コンポーネント
  - 3D インタラクション設計
  - ハプティックフィードバック

### 3. パフォーマンス最適化
- **レンダリング最適化**
  - インスタンシング
  - LOD (Level of Detail)
  - フラスタムカリング

- **メモリ管理**
  - ジオメトリとマテリアルの再利用
  - テクスチャアトラス
  - 不要オブジェクトの破棄

## 使用シーン

### いつ使うべきか
1. **3D UI フレームワーク開発時**
   - データ可視化アプリケーション
   - プロセス管理ダッシュボード
   - 複雑な情報の空間表現

2. **WebXR アプリケーション開発時**
   - VR/AR 対応 Web アプリ
   - 没入型データ探索ツール
   - 空間コンピューティング UI

3. **インタラクティブな 3D 体験を作成時**
   - リアルタイム 3D 可視化
   - ゲーム的 UI/UX
   - 教育/トレーニングアプリ

### 使わない方が良い場合
- 単純な 2D UI で十分な場合
- パフォーマンスが厳しく制限されているデバイス向け
- アクセシビリティが最優先の場合

## アーキテクチャ

```
┌─────────────────────────────────────────┐
│         Vue 3 Component Layer           │
│  (Process3DView.vue, Controls, etc.)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         3D Framework Layer              │
│  - LayoutManager (モード管理)            │
│  - TransitionOrchestrator (遷移制御)     │
│  - InteractionHandler (入力処理)         │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Three.js Core Layer             │
│  - Scene, Camera, Renderer              │
│  - Geometry, Material, Mesh             │
│  - Lights, Shadows, Post-processing     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         WebXR Integration Layer         │
│  - XRSession, XRFrame                   │
│  - Controller Input                     │
│  - Spatial Tracking                     │
└─────────────────────────────────────────┘
```

## コアコンセプト

### 1. Layout Manager
各表示モードのレイアウト計算とトランジションを一元管理

```typescript
class LayoutManager {
  switchMode(mode: ViewMode): void
  calculateLayouts(): void
  animateToTargets(): void
}
```

### 2. Process Node
3D 空間上のプロセス表現

```typescript
interface ProcessNode {
  id: string;
  mesh: THREE.Group;
  targetLayout: LayoutPosition;
  currentLayout: LayoutPosition;
}
```

### 3. View Mode
4 つの表示モードによるシームレスな UI 変化

- **List**: 情報密度高く一覧表示
- **Graph**: 関係性を視覚的に表現
- **Control**: 操作に特化したレイアウト
- **Edit**: 詳細編集に最適化

## 開発ガイドライン

### パフォーマンス

1. **60 FPS を維持**
   - requestAnimationFrame の適切な使用
   - 重い処理は Web Worker へ
   - レンダリングは必要な時のみ

2. **メモリ使用量の管理**
   - 不要なオブジェクトは dispose()
   - テクスチャサイズの最適化
   - ジオメトリの共有

### アクセシビリティ

1. **代替手段の提供**
   - 2D フォールバック UI
   - キーボード操作対応
   - スクリーンリーダー対応

2. **快適性への配慮**
   - VR 酔い軽減（滑らかな動き）
   - 視線移動の最小化
   - 適切なコントラスト

### コード品質

1. **型安全性**
   - TypeScript での厳密な型定義
   - Three.js 型定義の活用

2. **テスト**
   - ユニットテスト（ビジネスロジック）
   - ビジュアルリグレッションテスト
   - パフォーマンステスト

## リファレンス

詳細な実装パターンとベストプラクティスは以下を参照：

- [3D UI Patterns](./reference/3d_ui_patterns.md)
- [WebXR Integration](./reference/webxr_integration.md)
- [Performance Optimization](./reference/performance_optimization.md)
- [Best Practices](./reference/best_practices.md)

## サンプルコード

### 基本的な 3D シーン構築

```typescript
import * as THREE from 'three';
import { LayoutManager } from '@/lib/3d/LayoutManager';

// シーンのセットアップ
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ antialias: true });

// レイアウトマネージャーの初期化
const layoutManager = new LayoutManager();

// プロセスノードの作成と登録
processes.forEach(process => {
  const node = createProcessNode(process);
  scene.add(node);
  layoutManager.registerNode(process.id, node);
});

// モード切り替え
layoutManager.switchMode('graph');

// アニメーションループ
function animate() {
  requestAnimationFrame(animate);
  layoutManager.update();
  renderer.render(scene, camera);
}
```

### WebXR セッション開始

```typescript
// XR ボタンの作成
const button = VRButton.createButton(renderer);
document.body.appendChild(button);

// XR セッション開始時の処理
renderer.xr.addEventListener('sessionstart', () => {
  console.log('VR セッション開始');
});

// XR 有効化
renderer.xr.enabled = true;
```

## 今後の拡張

- [ ] Hand Tracking API 統合
- [ ] AR マーカー認識
- [ ] マルチユーザー同期
- [ ] 物理エンジン統合（Cannon.js / Ammo.js）
- [ ] リアルタイムグローバルイルミネーション
- [ ] AI アシスタント統合（音声コントロール）

## 関連リンク

- [Three.js Documentation](https://threejs.org/docs/)
- [WebXR Device API](https://immersiveweb.dev/)
- [A-Frame](https://aframe.io/) - 参考実装
- [Babylon.js](https://www.babylonjs.com/) - 代替フレームワーク
