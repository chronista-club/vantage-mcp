# visionOS WebXR スキル

このスキルは、Babylon.jsを使用してApple Vision Pro向けのWebXR体験を構築する際の必須知識とベストプラクティスを提供します。

## 対応状況（2024-2025年）

### ✅ 本番利用可能
- **Babylon.js 7.0以降**: 正式にVision Pro WebXR対応
- **対応モード**: `immersive-vr`のみ（`immersive-ar`は未対応）
- **必要環境**: Safari for visionOS + 実験的WebXR機能の有効化

### ⚠️ 制限事項
1. **ARモード未対応**: Vision Proは本来AR特化デバイスだが、WebXRではVRモードでのみ動作
2. **パフォーマンス問題**: 一部シーンでクラッシュの可能性
3. **テレポーテーション**: 床メッシュ設定後も一部環境で動作不良
4. **色再現**: シミュレータで色が薄く見える報告あり

## 必須実装パターン

### 1. WebXRセッション初期化

```typescript
import * as BABYLON from '@babylonjs/core';

// シーンのパフォーマンス優先度を設定（クラッシュ対策）
scene.performancePriority = BABYLON.ScenePerformancePriority.BackwardCompatible;

// WebXR Experience Helper作成
const xrHelper = await scene.createDefaultXRExperienceAsync({
  // immersive-vr モードのみ対応
  uiOptions: {
    sessionMode: 'immersive-vr'
  }
});
```

### 2. Vision Pro特有の機能対応

```typescript
// ハンドトラッキングとコントローラーの同時使用
if (xrHelper.baseExperience) {
  // ハンドトラッキング有効化
  const handTracking = xrHelper.baseExperience.featuresManager.enableFeature(
    BABYLON.WebXRFeatureName.HAND_TRACKING,
    'latest'
  );
  
  // コントローラー入力も同時に利用可能
  xrHelper.input.onControllerAddedObservable.add((controller) => {
    controller.onMotionControllerInitObservable.add((motionController) => {
      // コントローラー操作の実装
    });
  });
}
```

### 3. パフォーマンス最適化

```typescript
// アンチエイリアス付きマルチビュー対応
engine.setHardwareScalingLevel(1.0); // Vision Proの高解像度に対応

// レンダリング最適化
scene.autoClear = false;
scene.autoClearDepthAndStencil = false;

// オクルージョンクエリの無効化（Vision Pro用）
scene.blockMaterialDirtyMechanism = true;
```

### 4. テレポーテーション実装

```typescript
// 床メッシュの定義
const ground = BABYLON.MeshBuilder.CreateGround('ground', {
  width: 10,
  height: 10
}, scene);

// テレポーテーション機能の有効化
const teleportation = xrHelper.baseExperience.featuresManager.enableFeature(
  BABYLON.WebXRFeatureName.TELEPORTATION,
  'stable',
  {
    xrInput: xrHelper.input,
    floorMeshes: [ground], // 床メッシュを明示的に指定
    snapPositions: [], // スナップポイント（任意）
    defaultTargetMeshOptions: {
      teleportationFillColor: '#3e4a5d',
      teleportationBorderColor: '#ffffff'
    }
  }
);
```

### 5. UIとインタラクション

```typescript
import { AdvancedDynamicTexture, Button } from '@babylonjs/gui';

// 3D空間UIの作成
const advancedTexture = AdvancedDynamicTexture.CreateFullscreenUI('UI');

// タッチ可能なボタン
const button = Button.CreateSimpleButton('button', 'Click Me');
button.width = '200px';
button.height = '80px';
button.color = 'white';
button.background = 'blue';
button.onPointerClickObservable.add(() => {
  console.log('Button clicked in VR!');
});

advancedTexture.addControl(button);

// ワールドスケールの調整
const worldScale = 1.0; // Vision Proのスケールに合わせる
scene.getTransformNodes().forEach(node => {
  node.scaling.scaleInPlace(worldScale);
});
```

## デバッグとテスト

### Safari設定（Vision Pro実機）

1. **設定 > Safari > 詳細**
2. **「実験的な機能」を有効化**
3. **「WebXR Device API」を有効化**

### WebXRシミュレータ（開発時）

```typescript
// ブラウザ開発者ツールでWebXRエミュレータ拡張を使用
// Chrome: WebXR API Emulator
// 注意: 色再現に問題があるため、実機テスト必須
```

### パフォーマンス計測

```typescript
scene.onBeforeRenderObservable.add(() => {
  const fps = engine.getFps();
  const drawCalls = scene.getActiveMeshes().length;
  
  if (fps < 72) { // Vision Proの推奨FPS: 90Hz
    console.warn(`Low FPS: ${fps}, Draw calls: ${drawCalls}`);
  }
});
```

## トラブルシューティング

### クラッシュが発生する場合

```typescript
// 1. パフォーマンス優先度を変更
scene.performancePriority = BABYLON.ScenePerformancePriority.BackwardCompatible;

// 2. シャドウを無効化
shadowGenerator.enabled = false;

// 3. ポストプロセスを削減
scene.postProcesses = [];
```

### テレポーテーションが動作しない場合

```typescript
// 床メッシュのメタデータを設定
ground.metadata = { isFloor: true };

// コリジョン設定
ground.checkCollisions = true;

// 再度テレポーテーション機能を有効化
teleportation.detach();
teleportation.attach();
```

### 色が薄く見える場合

```typescript
// カラースペース調整（実験的）
scene.imageProcessingConfiguration.toneMappingEnabled = true;
scene.imageProcessingConfiguration.toneMappingType = BABYLON.ImageProcessingConfiguration.TONEMAPPING_ACES;
scene.imageProcessingConfiguration.exposure = 1.2;
```

## ベストプラクティス

### 1. プログレッシブエンハンスメント

```typescript
// Vision Pro検出
const isVisionPro = navigator.userAgent.includes('Vision');

if (isVisionPro) {
  // Vision Pro最適化パスを使用
  engine.setHardwareScalingLevel(1.0);
} else {
  // 通常のWebXRデバイス用設定
  engine.setHardwareScalingLevel(0.8);
}
```

### 2. フォールバック実装

```typescript
async function initXR() {
  try {
    // WebXR対応チェック
    const supported = await BABYLON.WebXRSessionManager.IsSessionSupportedAsync('immersive-vr');
    
    if (!supported) {
      console.warn('WebXR not supported, falling back to 3D view');
      // 通常の3Dビューワーとして動作
      return;
    }
    
    const xrHelper = await scene.createDefaultXRExperienceAsync({
      uiOptions: { sessionMode: 'immersive-vr' }
    });
    
  } catch (error) {
    console.error('WebXR initialization failed:', error);
    // フォールバック処理
  }
}
```

### 3. メモリ管理

```typescript
// シーン破棄時のクリーンアップ
scene.onDisposeObservable.add(() => {
  xrHelper?.dispose();
  engine.dispose();
});

// 定期的なガベージコレクション
setInterval(() => {
  scene.cleanCachedTextureBuffer();
}, 60000);
```

## 参考リンク

- [Babylon.js 7.0 公式発表](https://babylonjs.medium.com/introducing-babylon-js-7-0-a141cd7ede0d)
- [Babylon.js WebXR ドキュメント](https://doc.babylonjs.com/features/featuresDeepDive/webXR)
- [Vision Pro WebXR フォーラム](https://forum.babylonjs.com/t/webxr-in-the-apple-vision-pro/46253)
- [Apple WebXR ドキュメント](https://developer.apple.com/documentation/visionos)

## 今後の展望

- **immersive-ar モード**: Appleによる将来的なサポートが期待される
- **Passthrough API**: より自然なAR体験のための透過表示
- **Spatial Audio**: 空間オーディオの完全統合
- **Eye Tracking**: 視線追跡APIの公開（プライバシー配慮が必要）

---

**重要**: このスキルは2024-2025年時点の情報です。Babylon.jsとvisionOSは急速に進化しているため、最新のドキュメントも併せて確認してください。
