import * as THREE from "three";
import * as TWEEN from "@tweenjs/tween.js";

export type ViewMode = "list" | "graph" | "control" | "edit";

export interface LayoutPosition {
  position: THREE.Vector3;
  rotation: THREE.Euler;
  scale: THREE.Vector3;
}

export interface ProcessNode {
  id: string;
  mesh: THREE.Group;
  targetLayout: LayoutPosition;
  currentLayout: LayoutPosition;
}

/**
 * 3D Layout Manager
 * 各表示モードのレイアウト計算とトランジションを管理
 */
export class LayoutManager {
  private nodes: Map<string, ProcessNode> = new Map();
  private currentMode: ViewMode = "list";
  private transitionDuration = 800; // ms

  /**
   * ノードを登録
   */
  registerNode(id: string, mesh: THREE.Group): void {
    const initialLayout: LayoutPosition = {
      position: new THREE.Vector3(0, 0, 0),
      rotation: new THREE.Euler(0, 0, 0),
      scale: new THREE.Vector3(1, 1, 1),
    };

    this.nodes.set(id, {
      id,
      mesh,
      targetLayout: { ...initialLayout },
      currentLayout: { ...initialLayout },
    });
  }

  /**
   * ノードを削除
   */
  removeNode(id: string): void {
    this.nodes.delete(id);
  }

  /**
   * モード切り替え
   */
  switchMode(mode: ViewMode): void {
    this.currentMode = mode;
    this.calculateLayouts();
    this.animateToTargets();
  }

  /**
   * 各モードのレイアウトを計算
   */
  private calculateLayouts(): void {
    const nodeArray = Array.from(this.nodes.values());

    switch (this.currentMode) {
      case "list":
        this.calculateListLayout(nodeArray);
        break;
      case "graph":
        this.calculateGraphLayout(nodeArray);
        break;
      case "control":
        this.calculateControlLayout(nodeArray);
        break;
      case "edit":
        this.calculateEditLayout(nodeArray);
        break;
    }
  }

  /**
   * リストモードのレイアウト
   * 垂直に整列した3Dカードリスト
   */
  private calculateListLayout(nodes: ProcessNode[]): void {
    const spacing = 3.5;
    const startY = (nodes.length - 1) * spacing * 0.5;

    nodes.forEach((node, index) => {
      node.targetLayout.position.set(0, startY - index * spacing, 0);
      node.targetLayout.rotation.set(0, 0, 0);
      node.targetLayout.scale.set(4, 2, 0.2);
    });
  }

  /**
   * グラフモードのレイアウト
   * 円形または物理演算ベースの3Dネットワーク
   */
  private calculateGraphLayout(nodes: ProcessNode[]): void {
    const radius = 8;
    const angleStep = (Math.PI * 2) / nodes.length;

    nodes.forEach((node, index) => {
      const angle = index * angleStep;
      const x = Math.cos(angle) * radius;
      const z = Math.sin(angle) * radius;
      const y = Math.sin(index * 0.5) * 2; // 波のような高さ

      node.targetLayout.position.set(x, y, z);
      node.targetLayout.rotation.set(0, -angle, 0);
      node.targetLayout.scale.set(1.5, 1.5, 1.5);
    });
  }

  /**
   * コントロールモードのレイアウト
   * グリッド配置で手前に展開
   */
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
        2, // 手前に
      );
      node.targetLayout.rotation.set(0, 0, 0);
      node.targetLayout.scale.set(3, 2.5, 0.3);
    });
  }

  /**
   * エディットモードのレイアウト
   * 選択されたノードを中央に拡大、他は背景に
   */
  private calculateEditLayout(nodes: ProcessNode[]): void {
    // 最初のノードを編集対象として中央に配置（実際は選択されたノードを使用）
    nodes.forEach((node, index) => {
      if (index === 0) {
        // 編集対象
        node.targetLayout.position.set(0, 0, 0);
        node.targetLayout.rotation.set(0, 0, 0);
        node.targetLayout.scale.set(6, 4, 0.5);
      } else {
        // 背景のノード
        const angle = ((index - 1) / (nodes.length - 1)) * Math.PI * 2;
        const radius = 12;
        node.targetLayout.position.set(
          Math.cos(angle) * radius,
          Math.sin(angle) * radius,
          -5, // 奥に
        );
        node.targetLayout.rotation.set(0, 0, 0);
        node.targetLayout.scale.set(1, 1, 0.1);
      }
    });
  }

  /**
   * ターゲットレイアウトへアニメーション
   */
  private animateToTargets(): void {
    this.nodes.forEach((node) => {
      // Position tween
      new TWEEN.Tween(node.mesh.position)
        .to(
          {
            x: node.targetLayout.position.x,
            y: node.targetLayout.position.y,
            z: node.targetLayout.position.z,
          },
          this.transitionDuration,
        )
        .easing(TWEEN.Easing.Cubic.InOut)
        .start();

      // Rotation tween
      new TWEEN.Tween(node.mesh.rotation)
        .to(
          {
            x: node.targetLayout.rotation.x,
            y: node.targetLayout.rotation.y,
            z: node.targetLayout.rotation.z,
          },
          this.transitionDuration,
        )
        .easing(TWEEN.Easing.Cubic.InOut)
        .start();

      // Scale tween
      new TWEEN.Tween(node.mesh.scale)
        .to(
          {
            x: node.targetLayout.scale.x,
            y: node.targetLayout.scale.y,
            z: node.targetLayout.scale.z,
          },
          this.transitionDuration,
        )
        .easing(TWEEN.Easing.Cubic.InOut)
        .start();
    });
  }

  /**
   * アニメーションフレーム更新（毎フレーム呼ぶ）
   */
  update(): void {
    TWEEN.update();
  }

  /**
   * カメラ位置を現在のモードに最適化
   */
  getCameraPosition(mode: ViewMode): THREE.Vector3 {
    switch (mode) {
      case "list":
        return new THREE.Vector3(10, 0, 0); // 横から
      case "graph":
        return new THREE.Vector3(0, 5, 15); // 斜め上から
      case "control":
        return new THREE.Vector3(0, 0, 12); // 正面から
      case "edit":
        return new THREE.Vector3(0, 0, 10); // やや近くから正面
      default:
        return new THREE.Vector3(0, 0, 15);
    }
  }

  /**
   * カメラ視点を現在のモードに最適化
   */
  getCameraLookAt(_mode: ViewMode): THREE.Vector3 {
    // すべてのモードで原点を見る
    return new THREE.Vector3(0, 0, 0);
  }
}
