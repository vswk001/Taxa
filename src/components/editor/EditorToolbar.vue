<!-- src/components/editor/EditorToolbar.vue -->
<template>
  <div class="editor-toolbar">
    <button v-for="btn in buttons" :key="btn.action" @click="emit('action', btn.action)" :title="btn.label">
      {{ btn.icon }}
    </button>
  </div>
</template>

<script setup lang="ts">
const emit = defineEmits<{ action: [action: string] }>();

const buttons = [
  { action: 'bold', label: '粗体 (Ctrl+B)', icon: 'B' },
  { action: 'italic', label: '斜体 (Ctrl+I)', icon: 'I' },
  { action: 'heading', label: '标题', icon: 'H' },
  { action: 'list', label: '列表', icon: '•' },
  { action: 'quote', label: '引用', icon: '>' },
  { action: 'code', label: '代码', icon: '</>' },
  { action: 'link', label: '链接', icon: '🔗' },
  { action: 'image', label: '图片', icon: '🖼' },
];
</script>

<style scoped>
.editor-toolbar {
  display: flex; align-items: center; gap: 2px;
  padding: 6px 12px; border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}
.editor-toolbar button {
  width: 30px; height: 28px; font-size: 13px; border-radius: 4px;
  color: var(--text-secondary); display: flex; align-items: center; justify-content: center;
}
.editor-toolbar button:hover { background: var(--border-color); color: var(--text-primary); }
</style>
