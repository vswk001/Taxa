<template>
  <div class="graph-view">
    <canvas ref="canvasRef" class="graph-canvas" @mousedown="startDrag" @mousemove="onDrag" @mouseup="stopDrag" @mouseleave="stopDrag"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const canvasRef = ref<HTMLCanvasElement>();

interface GraphNode { id: string; title: string; folder: string; }
interface GraphEdge { source: string; target: string; }
interface GraphData { nodes: GraphNode[]; edges: GraphEdge[]; }

interface NodeWithPosition extends GraphNode {
  x: number;
  y: number;
}

let nodes: NodeWithPosition[] = [];
let edges: GraphEdge[] = [];
// id -> node map, rebuilt on load: edge drawing is O(E) instead of O(E×N).
let nodeMap = new Map<string, NodeWithPosition>();
let dragging: string | null = null;
let animationId: number | null = null;
let animating = false;

function resizeCanvas() {
  const canvas = canvasRef.value;
  if (!canvas || !canvas.parentElement) return false;
  // devicePixelRatio scaling keeps the canvas crisp on HiDPI screens; all
  // drawing coordinates stay in CSS pixels via the transform below.
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.parentElement.clientWidth * dpr;
  canvas.height = canvas.parentElement.clientHeight * dpr;
  const ctx = canvas.getContext('2d');
  if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  return true;
}

async function loadGraphData() {
  try {
    const data = await invoke<GraphData>('get_graph_data');
    if (!canvasRef.value || !resizeCanvas()) return;

    const width = canvasRef.value.parentElement!.clientWidth;
    const height = canvasRef.value.parentElement!.clientHeight;
    const cx = width / 2;
    const cy = height / 2;

    nodes = data.nodes.map((n, i) => {
      const angle = i * 2 * Math.PI / Math.max(data.nodes.length, 1);
      return { ...n, x: cx + Math.cos(angle) * Math.min(200, width / 3), y: cy + Math.sin(angle) * Math.min(200, height / 3) };
    });
    edges = data.edges;
    nodeMap = new Map(nodes.map(n => [n.id, n]));
    startAnimation();
  } catch (e) {
    console.error('Failed to load graph data:', e);
  }
}

function startAnimation() {
  if (animationId) cancelAnimationFrame(animationId);
  animating = true;
  draw();
}

function stopAnimation() {
  animating = false;
  if (animationId) {
    cancelAnimationFrame(animationId);
    animationId = null;
  }
}

function draw() {
  if (!animating) return;
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const styles = getComputedStyle(document.documentElement);
  const edgeColor = styles.getPropertyValue('--border-color').trim() || '#ccc';
  const nodeColor = styles.getPropertyValue('--accent-color').trim() || '#4a90d9';
  const textColor = styles.getPropertyValue('--text-secondary').trim() || '#333';
  const dpr = window.devicePixelRatio || 1;
  const width = canvas.width / dpr;
  const height = canvas.height / dpr;

  ctx.clearRect(0, 0, width, height);

  // Draw edges
  ctx.strokeStyle = edgeColor;
  ctx.lineWidth = 1;
  for (const e of edges) {
    const s = nodeMap.get(e.source);
    const t = nodeMap.get(e.target);
    if (s && t) {
      ctx.beginPath();
      ctx.moveTo(s.x, s.y);
      ctx.lineTo(t.x, t.y);
      ctx.stroke();
    }
  }

  // Draw nodes
  for (const n of nodes) {
    ctx.beginPath();
    ctx.arc(n.x, n.y, 6, 0, Math.PI * 2);
    ctx.fillStyle = nodeColor;
    ctx.fill();
    ctx.fillStyle = textColor;
    ctx.font = '11px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(n.title, n.x, n.y - 12);
  }

  animationId = requestAnimationFrame(draw);
}

function startDrag(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;
  dragging = nodes.find(n => Math.hypot(n.x - mx, n.y - my) < 10)?.id ?? null;
}

function onDrag(e: MouseEvent) {
  if (!dragging) return;
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const node = nodeMap.get(dragging);
  if (node) {
    node.x = e.clientX - rect.left;
    node.y = e.clientY - rect.top;
  }
}

function stopDrag() {
  dragging = null;
}

onMounted(() => {
  loadGraphData();
  window.addEventListener('resize', handleResize);
  document.addEventListener('visibilitychange', handleVisibilityChange);
});

onBeforeUnmount(() => {
  stopAnimation();
  window.removeEventListener('resize', handleResize);
  document.removeEventListener('visibilitychange', handleVisibilityChange);
});

function handleVisibilityChange() {
  if (document.hidden) {
    stopAnimation();
  } else if (nodes.length > 0) {
    startAnimation();
  }
}

function handleResize() {
  if (resizeCanvas() && nodes.length > 0) {
    startAnimation();
  }
}
</script>

<style scoped>
.graph-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.graph-canvas {
  flex: 1;
  width: 100%;
  cursor: grab;
}

.graph-canvas:active {
  cursor: grabbing;
}
</style>
