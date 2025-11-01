# 3D UI Design Patterns

本格的な 3DCG UI を構築するための設計パターン集です。

## 目次

1. [空間レイアウトパターン](#空間レイアウトパターン)
2. [インタラクションパターン](#インタラクションパターン)
3. [トランジションパターン](#トランジションパターン)
4. [データ可視化パターン](#データ可視化パターン)

---

## 空間レイアウトパターン

### 1. Vertical List Layout (垂直リストレイアウト)

**用途**: 情報を順序立てて表示する場合

**特徴**:
- カードが垂直方向に整列
- スクロールで上下移動
- 各カードは薄い直方体（Box Geometry）

**実装例**:
```typescript
private calculateListLayout(nodes: ProcessNode[]): void {
  const spacing = 3.5; // カード間の距離
  const startY = (nodes.length - 1) * spacing * 0.5; // 中央揃え

  nodes.forEach((node, index) => {
    node.targetLayout.position.set(
      0,                            // X: 中央
      startY - index * spacing,     // Y: 上から順に配置
      0                             // Z: 同じ深度
    );
    node.targetLayout.rotation.set(0, 0, 0);
    node.targetLayout.scale.set(4, 2, 0.2); // 横長カード
  });
}
```

**カメラ設定**:
```typescript
camera.position.set(10, 0, 0); // 横から見る視点
camera.lookAt(0, 0, 0);        // 原点を注視
```

**最適なユースケース**:
- プロセス一覧
- タスクリスト
- タイムライン表示

---

### 2. Circular Graph Layout (円形グラフレイアウト)

**用途**: ノード間の関係性を視覚化

**特徴**:
- ノードが円形に配置
- 中心からの距離で重要度を表現
- エッジ（線）で関係性を接続

**実装例**:
```typescript
private calculateGraphLayout(nodes: ProcessNode[]): void {
  const radius = 8;
  const angleStep = (Math.PI * 2) / nodes.length;

  nodes.forEach((node, index) => {
    const angle = index * angleStep;
    const x = Math.cos(angle) * radius;
    const z = Math.sin(angle) * radius;
    const y = Math.sin(index * 0.5) * 2; // 波状の高さ変化

    node.targetLayout.position.set(x, y, z);
    node.targetLayout.rotation.set(0, -angle, 0); // 中心を向く
    node.targetLayout.scale.set(1.5, 1.5, 1.5);   // 立方体
  });
}

// エッジの描画
private drawEdges(nodes: ProcessNode[]): void {
  nodes.forEach((nodeA, i) => {
    nodes.slice(i + 1).forEach(nodeB => {
      const geometry = new THREE.BufferGeometry().setFromPoints([
        nodeA.mesh.position,
        nodeB.mesh.position
      ]);
      const material = new THREE.LineBasicMaterial({ color: 0x444444 });
      const line = new THREE.Line(geometry, material);
      this.scene.add(line);
    });
  });
}
```

**カメラ設定**:
```typescript
camera.position.set(0, 5, 15); // 斜め上から俯瞰
camera.lookAt(0, 0, 0);
```

---

### 3. Grid Control Layout (グリッドコントロールレイアウト)

**用途**: 操作可能な要素を整理して配置

**特徴**:
- グリッド状に整列
- 手前に配置（Z 軸正方向）
- ホバー時に拡大

**実装例**:
```typescript
private calculateControlLayout(nodes: ProcessNode[]): void {
  const cols = Math.ceil(Math.sqrt(nodes.length));
  const spacing = 4;
  const offsetX = -((cols - 1) * spacing) / 2;
  const offsetY = -((Math.ceil(nodes.length / cols) - 1) * spacing) / 2;

  nodes.forEach((node, index) => {
    const col = index % cols;
    const row = Math.floor(index / cols);

    node.targetLayout.position.set(
      offsetX + col * spacing,
      offsetY + row * spacing,
      2 // 手前に配置
    );
    node.targetLayout.rotation.set(0, 0, 0);
    node.targetLayout.scale.set(3, 2.5, 0.3);
  });
}
```

**カメラ設定**:
```typescript
camera.position.set(0, 0, 12); // 正面から
camera.lookAt(0, 0, 0);
```

---

### 4. Focus + Context Layout (フォーカス + コンテキストレイアウト)

**用途**: 1 つの要素を編集しながら他の要素も見える状態を維持

**特徴**:
- 選択された要素が中央に拡大
- 他の要素は背景に小さく配置
- 深度（Z 軸）で階層を表現

**実装例**:
```typescript
private calculateEditLayout(nodes: ProcessNode[], selectedIndex: number = 0): void {
  nodes.forEach((node, index) => {
    if (index === selectedIndex) {
      // 編集対象: 中央に大きく
      node.targetLayout.position.set(0, 0, 0);
      node.targetLayout.rotation.set(0, 0, 0);
      node.targetLayout.scale.set(6, 4, 0.5);
    } else {
      // 背景: 円形に小さく配置
      const angle = ((index - (index > selectedIndex ? 1 : 0)) / (nodes.length - 1)) * Math.PI * 2;
      const radius = 12;
      
      node.targetLayout.position.set(
        Math.cos(angle) * radius,
        Math.sin(angle) * radius,
        -5 // 奥に配置
      );
      node.targetLayout.rotation.set(0, 0, 0);
      node.targetLayout.scale.set(1, 1, 0.1); // 薄く表示
    }
  });
}
```

---

## インタラクションパターン

### 1. Ray Casting Selection (レイキャスト選択)

**マウス/コントローラーからのレイキャストでオブジェクトを選択**

```typescript
class InteractionHandler {
  private raycaster = new THREE.Raycaster();
  private mouse = new THREE.Vector2();

  onMouseMove(event: MouseEvent, camera: THREE.Camera, objects: THREE.Object3D[]): void {
    // NDC 座標に変換 (-1 to +1)
    this.mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    this.mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

    // レイキャスト実行
    this.raycaster.setFromCamera(this.mouse, camera);
    const intersects = this.raycaster.intersectObjects(objects, true);

    if (intersects.length > 0) {
      const selected = intersects[0].object;
      this.onHover(selected);
    }
  }

  onHover(object: THREE.Object3D): void {
    // ホバー時の処理（拡大、発光など）
    new TWEEN.Tween(object.scale)
      .to({ x: 1.2, y: 1.2, z: 1.2 }, 200)
      .easing(TWEEN.Easing.Quadratic.Out)
      .start();
  }
}
```

---

### 2. Depth Press Animation (深度押し込みアニメーション)

**クリック時に要素が奥に押し込まれる触覚的フィードバック**

```typescript
onClick(object: THREE.Object3D): void {
  const originalZ = object.position.z;

  new TWEEN.Tween(object.position)
    .to({ z: originalZ - 0.5 }, 100) // 押し込み
    .easing(TWEEN.Easing.Quadratic.In)
    .onComplete(() => {
      // 元に戻る
      new TWEEN.Tween(object.position)
        .to({ z: originalZ }, 150)
        .easing(TWEEN.Easing.Elastic.Out)
        .start();
    })
    .start();
}
```

---

### 3. Orbit Camera Control (軌道カメラコントロール)

**マウスドラッグでシーンを回転**

```typescript
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';

const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;      // 慣性
controls.dampingFactor = 0.05;      // 減衰率
controls.minDistance = 5;           // 最小ズーム距離
controls.maxDistance = 50;          // 最大ズーム距離
controls.maxPolarAngle = Math.PI / 2; // 地平線より下に行かない

// アニメーションループ内で更新
function animate() {
  controls.update();
  renderer.render(scene, camera);
}
```

---

## トランジションパターン

### 1. Smooth Mode Transition (滑らかなモード遷移)

**TWEEN.js による位置・回転・スケールの同時アニメーション**

```typescript
animateToTargets(duration: number = 800): void {
  this.nodes.forEach((node) => {
    // Position
    new TWEEN.Tween(node.mesh.position)
      .to(node.targetLayout.position, duration)
      .easing(TWEEN.Easing.Cubic.InOut)
      .start();

    // Rotation
    new TWEEN.Tween(node.mesh.rotation)
      .to(node.targetLayout.rotation, duration)
      .easing(TWEEN.Easing.Cubic.InOut)
      .start();

    // Scale
    new TWEEN.Tween(node.mesh.scale)
      .to(node.targetLayout.scale, duration)
      .easing(TWEEN.Easing.Cubic.InOut)
      .start();
  });
}
```

---

### 2. Staggered Animation (時間差アニメーション)

**要素ごとに少しずつ遅延させて動的な印象を与える**

```typescript
animateStaggered(duration: number = 800, staggerDelay: number = 50): void {
  this.nodes.forEach((node, index) => {
    const delay = index * staggerDelay;

    new TWEEN.Tween(node.mesh.position)
      .to(node.targetLayout.position, duration)
      .delay(delay)
      .easing(TWEEN.Easing.Cubic.Out)
      .start();
  });
}
```

---

## データ可視化パターン

### 1. Color Mapping by State (状態による色分け)

**プロセス状態を色で視覚的に区別**

```typescript
getProcessColor(state: ProcessState): number {
  switch (state) {
    case 'running':
      return 0xdc26a0; // Creative マゼンタ
    case 'stopped':
      return 0x7decf2; // Provision シアン
    case 'failed':
      return 0xef6cdb; // Bradcast ピンク
    default:
      return 0x666666; // グレー
  }
}

// マテリアルに適用
const material = new THREE.MeshPhongMaterial({
  color: this.getProcessColor(process.state),
  emissive: this.getProcessColor(process.state),
  emissiveIntensity: 0.3,
});
```

---

### 2. Pulsing Effect for Running State (実行中のパルス効果)

**実行中のプロセスを脈動させて目立たせる**

```typescript
animate(): void {
  this.nodes.forEach((node) => {
    const process = this.getProcess(node.id);
    
    if (process.state === 'running') {
      const time = Date.now() * 0.003;
      const scale = 1 + Math.sin(time) * 0.1; // ±10% のスケール変化
      node.mesh.scale.set(scale, scale, scale);
    }
  });
}
```

---

### 3. Dynamic Text Labels (動的テキストラベル)

**Canvas を使って 3D 空間にテキストを描画**

```typescript
createTextSprite(text: string, size: number = 48): THREE.Sprite {
  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d')!;
  
  canvas.width = 512;
  canvas.height = 128;
  
  // 背景（オプション）
  context.fillStyle = 'rgba(0, 0, 0, 0.7)';
  context.fillRect(0, 0, canvas.width, canvas.height);
  
  // テキスト
  context.fillStyle = '#ffffff';
  context.font = `bold ${size}px sans-serif`;
  context.textAlign = 'center';
  context.textBaseline = 'middle';
  context.fillText(text, canvas.width / 2, canvas.height / 2);
  
  // スプライト作成
  const texture = new THREE.CanvasTexture(canvas);
  const material = new THREE.SpriteMaterial({ map: texture });
  const sprite = new THREE.Sprite(material);
  sprite.scale.set(4, 1, 1);
  
  return sprite;
}
```

---

## パフォーマンスのベストプラクティス

### 1. Object Pooling (オブジェクトプーリング)

**頻繁に作成/破棄するオブジェクトを再利用**

```typescript
class ObjectPool<T> {
  private pool: T[] = [];
  
  constructor(private factory: () => T, initialSize: number = 10) {
    for (let i = 0; i < initialSize; i++) {
      this.pool.push(factory());
    }
  }
  
  acquire(): T {
    return this.pool.pop() || this.factory();
  }
  
  release(object: T): void {
    this.pool.push(object);
  }
}
```

---

### 2. Frustum Culling (視錐台カリング)

**カメラに映っていないオブジェクトをスキップ**

```typescript
const frustum = new THREE.Frustum();
const cameraViewProjectionMatrix = new THREE.Matrix4();

function updateFrustum(camera: THREE.Camera): void {
  camera.updateMatrixWorld();
  cameraViewProjectionMatrix.multiplyMatrices(
    camera.projectionMatrix,
    camera.matrixWorldInverse
  );
  frustum.setFromProjectionMatrix(cameraViewProjectionMatrix);
}

function isVisible(object: THREE.Object3D): boolean {
  return frustum.intersectsObject(object);
}
```

---

## まとめ

これらのパターンを組み合わせることで、本格的な 3D UI フレームワークを構築できます。

**重要なポイント**:
- パフォーマンスを常に意識する（60 FPS 維持）
- ユーザー体験を最優先（滑らかな動き、分かりやすいフィードバック）
- アクセシビリティを忘れない（2D フォールバック、キーボード操作）
