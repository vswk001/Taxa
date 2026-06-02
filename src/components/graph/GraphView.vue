<!-- src/components/graph/GraphView.vue -->
<template>
  <div class="graph-view">
    <div class="graph-header">
      <span>笔记图谱</span>
      <button @click="emit('close')">×</button>
    </div>
    <canvas ref="canvasRef" class="graph-canvas" @mousedown="startDrag" @mousemove="onDrag" @mouseup="stopDrag"></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits<{ close: [] }>();
const canvasRef = ref<HTMLCanvasElement>();

interface GraphNode { id: string; title: string; folder: string; }
interface GraphEdge { source: string; target: string; }
interface GraphData { nodes: GraphNode[]; edges: GraphEdge[]; }

interface NodeWithPosition extends GraphNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

let nodes: NodeWithPosition[] = [];
let edges: GraphEdge[] = [];
let dragging: number | null = null;

onMounted(async () => {
  const data = await invoke<GraphData>('get_graph_data');
  const canvas = canvasRef.value!;
  // Set canvas size
  canvas.width = canvas.parentElement!.clientWidth;
  canvas.height = canvas.parentElement!.clientHeight - 50; // Subtract header height

  const cx = canvas.width / 2;
  const cy = canvas.height / 2;

  nodes = data.nodes.map((n, i) => {
    const angle = i * 2 * Math.PI / data.nodes.length;
    return {
      ...n,
      x: cx + Math.cos(angle) * 200,
      y: cy + Math.sin(angle) * 200,
      vx: 0,
      vy: 0,
    };
  });
  edges = data.edges;
  requestAnimationFrame(draw);
});

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d')!;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Draw edges
  ctx.strokeStyle = '#ccc';
  ctx.lineWidth = 1;
  for (const e of edges) {
    const s = nodes.find(n => n.id === e.source);
    const t = nodes.find(n => n.id === e.target);
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
    ctx.fillStyle = '#4a90d9';
    ctx.fill();
    ctx.fillStyle = '#333';
    ctx.font = '11px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(n.title, n.x, n.y - 12);
  }
  requestAnimationFrame(draw);
}

function startDrag(e: MouseEvent) {
  const canvas = canvasRef.value!;
  const rect = canvas.getBoundingClientRect();
  const mx = e.clientX - rect.left;
  const my = e.clientY - rect.top;
  dragging = nodes.findIndex(n => Math.hypot(n.x - mx, n.y - my) < 10);
}

function onDrag(e: MouseEvent) {
  if (dragging === null || dragging < 0) return;
  const canvas = canvasRef.value!;
  const rect = canvas.getBoundingClientRect();
  nodes[dragging].x = e.clientX - rect.left;
  nodes[dragging].y = e.clientY - rect.top;
}

function stopDrag() { dragging = null; }
</script>

<style scoped>
.graph-view {
  position: fixed;
  inset: 0;
  background: var(--bg-primary);
  z-index: 50;
  display: flex;
  flex-direction: column;
}

.graph-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.graph-header button {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-primary);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.graph-header button:hover {
  background: var(--hover-bg);
  border-radius: 4px;
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
