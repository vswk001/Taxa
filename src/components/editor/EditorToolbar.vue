<template>
  <div class="editor-toolbar">
    <button
      v-for="btn in buttons"
      :key="btn.action"
      @click="handleClick(btn.action)"
      :title="btn.label"
      class="toolbar-btn"
    >
      <span class="btn-icon">{{ btn.icon }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { defineEmits } from 'vue';

const emit = defineEmits<{ action: [action: string] }>();

const buttons = [
  { action: 'bold', label: '粗体 (Ctrl+B)', icon: 'B' },
  { action: 'italic', label: '斜体 (Ctrl+I)', icon: 'I' },
  { action: 'heading', label: '标题', icon: 'H' },
  { action: 'list', label: '列表', icon: '•' },
  { action: 'quote', label: '引用', icon: '›' },
  { action: 'code', label: '代码', icon: '<>' },
  { action: 'link', label: '链接', icon: '🔗' },
  { action: 'image', label: '图片', icon: '🖼' },
];

function handleClick(action: string) {
  emit('action', action);
}
</script>

<style scoped>
.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  overflow-x: auto;
}

.toolbar-btn {
  width: 32px;
  height: 28px;
  font-size: 13px;
  border-radius: 4px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.toolbar-btn:hover {
  background: var(--border-color);
  color: var(--text-primary);
  border-color: var(--border-color);
}

.toolbar-btn:active {
  transform: scale(0.95);
}

.btn-icon {
  font-weight: 600;
  user-select: none;
}
</style>
