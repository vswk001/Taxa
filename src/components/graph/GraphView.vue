<template>
  <div class="graph-view">
    <div class="graph-toolbar">
      <button :title="t('graph.zoomIn')" @click="zoomBy(1.2)">＋</button>
      <button :title="t('graph.zoomOut')" @click="zoomBy(1 / 1.2)">－</button>
      <button :title="t('graph.resetView')" @click="resetView">{{ Math.round(scale * 100) }}%</button>
    </div>
    <canvas
      ref="canvasRef"
      class="graph-canvas"
      @mousedown="onMouseDown"
      @mousemove="onDrag"
      @mouseup="onMouseUp"
      @mouseleave="stopInteraction"
    ></canvas>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';

const { t } = useI18n();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
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
// id -> node map, rebuilt on load: edge drawing and hit tests are O(1)/O(E).
let nodeMap = new Map<string, NodeWithPosition>();

// View transform: world coords -> screen = world * scale + offset.
let scale = 1;
let offsetX = 0;
let offsetY = 0;

// Interaction state: dragging a node, panning the canvas, or neither.
let dragging: string | null = null;
let panning = false;
let lastX = 0;
let lastY = 0;
let downX = 0;
let downY = 0;
let animationId: number | null = null;
let animating = false;

const CLICK_THRESHOLD = 4; // px of movement before a press counts as a drag

function toWorld(clientX: number, clientY: number): { x: number; y: number } {
  const rect = canvasRef.value!.getBoundingClientRect();
  return {
    x: (clientX - rect.left - offsetX) / scale,
    y: (clientY - rect.top - offsetY) / scale,
  };
}

function hitNode(clientX: number, clientY: number): NodeWithPosition | null {
  const { x, y } = toWorld(clientX, clientY);
  return nodes.find((n) => Math.hypot(n.x - x, n.y - y) < 12 / Math.max(scale, 0.2) + 6) ?? null;
}

function resizeCanvas(): boolean {
  const canvas = canvasRef.value;
  if (!canvas || !canvas.parentElement) return false;
  // devicePixelRatio scaling keeps the canvas crisp on HiDPI screens.
  const dpr = window.devicePixelRatio || 1;
  canvas.width = canvas.parentElement.clientWidth * dpr;
  canvas.height = canvas.parentElement.clientHeight * dpr;
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
      const angle = (i * 2 * Math.PI) / Math.max(data.nodes.length, 1);
      return {
        ...n,
        x: cx + Math.cos(angle) * Math.min(200, width / 3),
        y: cy + Math.sin(angle) * Math.min(200, height / 3),
      };
    });
    edges = data.edges;
    nodeMap = new Map(nodes.map((n) => [n.id, n]));
    resetView();
  } catch (e) {
    console.error('Failed to load graph data:', e);
  }
}

function resetView() {
  scale = 1;
  offsetX = 0;
  offsetY = 0;
  startAnimation();
}

function zoomBy(factor: number) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  zoomAt(rect.width / 2, rect.height / 2, factor);
  startAnimation();
}

/** Zoom around a screen-space point so the content under the cursor stays put. */
function zoomAt(screenX: number, screenY: number, factor: number) {
  const next = Math.min(3, Math.max(0.2, scale * factor));
  const worldX = (screenX - offsetX) / scale;
  const worldY = (screenY - offsetY) / scale;
  scale = next;
  offsetX = screenX - worldX * scale;
  offsetY = screenY - worldY * scale;
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const rect = canvasRef.value!.getBoundingClientRect();
  const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;
  zoomAt(e.clientX - rect.left, e.clientY - rect.top, factor);
  startAnimation();
}

function onMouseDown(e: MouseEvent) {
  const hit = hitNode(e.clientX, e.clientY);
  downX = e.clientX;
  downY = e.clientY;
  lastX = e.clientX;
  lastY = e.clientY;
  if (hit) {
    dragging = hit.id;
    panning = false;
  } else {
    dragging = null;
    panning = true;
  }
}

function onDrag(e: MouseEvent) {
  if (dragging) {
    const node = nodeMap.get(dragging);
    if (node) {
      node.x += (e.clientX - lastX) / scale;
      node.y += (e.clientY - lastY) / scale;
    }
    lastX = e.clientX;
    lastY = e.clientY;
  } else if (panning) {
    offsetX += e.clientX - lastX;
    offsetY += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
  }
}

async function onMouseUp(e: MouseEvent) {
  const moved = Math.hypot(e.clientX - downX, e.clientY - downY);
  const wasDraggingNode = dragging;
  stopInteraction();
  // A press-release without movement on a node is a click: open the note.
  if (wasDraggingNode && moved < CLICK_THRESHOLD) {
    const note = notebookStore.notes.find((n) => n.id === wasDraggingNode);
    if (note) {
      await notebookStore.openNote(note.id);
      editorStore.openTab(note.id, note.title);
    }
  }
}

function stopInteraction() {
  dragging = null;
  panning = false;
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

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  ctx.setTransform(dpr * scale, 0, 0, dpr * scale, dpr * offsetX, dpr * offsetY);

  // Edges
  ctx.strokeStyle = edgeColor;
  ctx.lineWidth = 1 / scale;
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

  // Nodes + labels (labels shrink out below reasonable zoom)
  for (const n of nodes) {
    ctx.beginPath();
    ctx.arc(n.x, n.y, 6, 0, Math.PI * 2);
    ctx.fillStyle = nodeColor;
    ctx.fill();
    if (scale > 0.45) {
      ctx.fillStyle = textColor;
      ctx.font = `${Math.max(9, 11 / scale)}px sans-serif`;
      ctx.textAlign = 'center';
      ctx.fillText(n.title, n.x, n.y - 12 / scale);
    }
  }

  animationId = requestAnimationFrame(draw);
}

onMounted(() => {
  loadGraphData();
  // Non-passive wheel listener so preventDefault stops page scroll.
  canvasRef.value?.addEventListener('wheel', onWheel, { passive: false });
  window.addEventListener('resize', handleResize);
  document.addEventListener('visibilitychange', handleVisibilityChange);
});

onBeforeUnmount(() => {
  stopAnimation();
  canvasRef.value?.removeEventListener('wheel', onWheel);
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
  position: relative;
}

.graph-toolbar {
  position: absolute;
  top: 10px;
  right: 12px;
  z-index: 10;
  display: flex;
  gap: 4px;
}

.graph-toolbar button {
  min-width: 34px;
  padding: 4px 8px;
  font-size: 13px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-primary);
}

.graph-toolbar button:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
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
