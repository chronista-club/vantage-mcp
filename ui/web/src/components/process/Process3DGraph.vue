<template>
  <div ref="containerRef" class="process-3d-graph"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import * as THREE from 'three';
import type { Process } from '@/types';
import { isRunning, isStopped, isFailed } from '@/types';

interface Props {
  processes: Process[];
}

const props = defineProps<Props>();

const containerRef = ref<HTMLDivElement | null>(null);

let scene: THREE.Scene;
let camera: THREE.PerspectiveCamera;
let renderer: THREE.WebGLRenderer;
let processNodes: Map<string, THREE.Mesh> = new Map();
let animationId: number;

// Mouse interaction
let mouseX = 0;
let mouseY = 0;
let targetRotationX = 0;
let targetRotationY = 0;
let currentRotationX = 0;
let currentRotationY = 0;

const initThreeJS = () => {
  if (!containerRef.value) return;

  // Scene setup
  scene = new THREE.Scene();
  scene.background = new THREE.Color(0x0a0a0a);

  // Camera setup
  const width = containerRef.value.clientWidth;
  const height = containerRef.value.clientHeight;
  camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
  camera.position.z = 15;

  // Renderer setup
  renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(width, height);
  renderer.setPixelRatio(window.devicePixelRatio);
  containerRef.value.appendChild(renderer.domElement);

  // Lighting
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
  scene.add(ambientLight);

  const pointLight = new THREE.PointLight(0xffffff, 1);
  pointLight.position.set(10, 10, 10);
  scene.add(pointLight);

  // Grid helper
  const gridHelper = new THREE.GridHelper(20, 20, 0x444444, 0x222222);
  scene.add(gridHelper);

  // Create process nodes
  createProcessNodes();

  // Start animation loop
  animate();
};

const createProcessNodes = () => {
  // Clear existing nodes
  processNodes.forEach((mesh) => {
    scene.remove(mesh);
  });
  processNodes.clear();

  // Create nodes for each process
  props.processes.forEach((process, index) => {
    const geometry = new THREE.BoxGeometry(1.5, 1.5, 1.5);
    const material = new THREE.MeshPhongMaterial({
      color: getProcessColor(process),
      emissive: getProcessEmissive(process),
      emissiveIntensity: 0.3,
      shininess: 100,
    });

    const mesh = new THREE.Mesh(geometry, material);

    // Position nodes in a circle
    const radius = 5;
    const angle = (index / props.processes.length) * Math.PI * 2;
    mesh.position.x = Math.cos(angle) * radius;
    mesh.position.z = Math.sin(angle) * radius;
    mesh.position.y = 0;

    // Add text label (using sprite)
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d')!;
    canvas.width = 512;
    canvas.height = 128;
    context.fillStyle = '#ffffff';
    context.font = 'bold 48px sans-serif';
    context.textAlign = 'center';
    context.fillText(process.id, 256, 80);

    const texture = new THREE.CanvasTexture(canvas);
    const spriteMaterial = new THREE.SpriteMaterial({ map: texture });
    const sprite = new THREE.Sprite(spriteMaterial);
    sprite.scale.set(4, 1, 1);
    sprite.position.y = 2;
    mesh.add(sprite);

    scene.add(mesh);
    processNodes.set(process.id, mesh);
  });
};

const getProcessColor = (process: Process): number => {
  if (isRunning(process.state)) return 0xdc26a0; // Creative magenta
  if (isFailed(process.state)) return 0xef6cdb;   // Broadcast pink
  if (isStopped(process.state)) return 0x7decf2; // Provision cyan
  return 0x666666; // Gray for not started
};

const getProcessEmissive = (process: Process): number => {
  if (isRunning(process.state)) return 0xdc26a0;
  if (isFailed(process.state)) return 0xef6cdb;
  return 0x000000;
};

const animate = () => {
  animationId = requestAnimationFrame(animate);

  // Smooth rotation based on mouse position
  currentRotationX += (targetRotationX - currentRotationX) * 0.05;
  currentRotationY += (targetRotationY - currentRotationY) * 0.05;

  // Apply rotation to scene
  scene.rotation.y = currentRotationY;
  scene.rotation.x = currentRotationX;

  // Animate individual nodes
  processNodes.forEach((mesh, id) => {
    mesh.rotation.x += 0.01;
    mesh.rotation.y += 0.01;

    // Pulsing effect for running processes
    const process = props.processes.find((p) => p.id === id);
    if (process && isRunning(process.state)) {
      const scale = 1 + Math.sin(Date.now() * 0.003) * 0.1;
      mesh.scale.set(scale, scale, scale);
    }
  });

  renderer.render(scene, camera);
};

const onMouseMove = (event: MouseEvent) => {
  if (!containerRef.value) return;

  const rect = containerRef.value.getBoundingClientRect();
  mouseX = ((event.clientX - rect.left) / rect.width) * 2 - 1;
  mouseY = -((event.clientY - rect.top) / rect.height) * 2 + 1;

  targetRotationY = mouseX * Math.PI;
  targetRotationX = mouseY * Math.PI * 0.5;
};

const onResize = () => {
  if (!containerRef.value) return;

  const width = containerRef.value.clientWidth;
  const height = containerRef.value.clientHeight;

  camera.aspect = width / height;
  camera.updateProjectionMatrix();

  renderer.setSize(width, height);
};

// Watch for process changes
watch(
  () => props.processes,
  () => {
    createProcessNodes();
  },
  { deep: true }
);

onMounted(() => {
  initThreeJS();
  window.addEventListener('resize', onResize);
  if (containerRef.value) {
    containerRef.value.addEventListener('mousemove', onMouseMove);
  }
});

onUnmounted(() => {
  cancelAnimationFrame(animationId);
  window.removeEventListener('resize', onResize);
  if (containerRef.value) {
    containerRef.value.removeEventListener('mousemove', onMouseMove);
  }
  if (renderer) {
    renderer.dispose();
  }
});
</script>

<style scoped lang="scss">
.process-3d-graph {
  width: 100%;
  height: 600px;
  border-radius: 12px;
  overflow: hidden;
  position: relative;
  background: oklch(0.08 0 0);

  @media (prefers-color-scheme: light) {
    background: oklch(0.12 0 0);
  }
}
</style>
