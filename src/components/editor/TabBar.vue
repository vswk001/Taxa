<!-- src/components/editor/TabBar.vue -->
<template>
  <div class="tab-bar">
    <div
      v-for="tab in editorStore.openTabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === editorStore.activeTabId }"
      @click="editorStore.activeTabId = tab.id"
    >
      <span class="tab-title">{{ tab.title }}</span>
      <button class="tab-close" @click.stop="editorStore.closeTab(tab.id)">×</button>
    </div>
    <button class="tab-new" @click="notebookStore.createNote(notebookStore.currentFolder || '未分类', '新笔记', '')">+</button>
  </div>
</template>

<script setup lang="ts">
import { useEditorStore } from '@/stores/editor';
import { useNotebookStore } from '@/stores/notebook';

const editorStore = useEditorStore();
const notebookStore = useNotebookStore();
</script>

<style scoped>
.tab-bar {
  display: flex; align-items: center; height: var(--tab-height);
  background: var(--bg-secondary); border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
}
.tab {
  display: flex; align-items: center; gap: 6px;
  padding: 0 12px; height: 100%; font-size: 13px;
  border-right: 1px solid var(--border-color); cursor: pointer;
  color: var(--text-secondary); white-space: nowrap;
}
.tab:hover { background: var(--bg-primary); }
.tab.active { background: var(--bg-primary); color: var(--text-primary); border-bottom: 2px solid var(--accent-color); }
.tab-close { font-size: 16px; color: var(--text-secondary); padding: 0 2px; }
.tab-close:hover { color: var(--danger-color); }
.tab-new { padding: 0 12px; font-size: 18px; color: var(--text-secondary); height: 100%; }
.tab-new:hover { color: var(--text-primary); }
</style>
