<template>
  <div class="tab-bar">
    <div
      v-for="tab in editorStore.openTabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === editorStore.activeTabId }"
      @click="handleTabClick(tab.id)"
    >
      <span class="tab-title">{{ tab.title }}</span>
      <button class="tab-close" @click.stop="handleCloseTab(tab.id)" title="关闭标签">×</button>
    </div>
    <button class="tab-new" @click="handleNewNote" title="新建笔记">+</button>
  </div>
</template>

<script setup lang="ts">
import { useEditorStore } from '@/stores/editor';
import { useNotebookStore } from '@/stores/notebook';

const editorStore = useEditorStore();
const notebookStore = useNotebookStore();

function handleTabClick(tabId: string) {
  if (editorStore.activeTabId === tabId) return;
  editorStore.setActiveTab(tabId);
}

async function handleNewNote() {
  try {
    const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');
    await notebookStore.createNote(folder, '新笔记', '');
  } catch (error) {
    console.error('Failed to create new note:', error);
  }
}

function handleCloseTab(tabId: string) {
  editorStore.closeTab(tabId);

  if (editorStore.activeTabId && editorStore.activeTabId !== '__graph__') {
    notebookStore.openNote(editorStore.activeTabId);
  } else if (!editorStore.activeTabId) {
    notebookStore.currentNote = null;
  }
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: 40px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
  overflow-y: hidden;
}

.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 100%;
  font-size: 13px;
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  color: var(--text-secondary);
  white-space: nowrap;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.tab:hover {
  background: var(--bg-primary);
}

.tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-color);
}

.tab-title {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  font-size: 18px;
  color: var(--text-secondary);
  padding: 0 2px;
  background: none;
  border: none;
  cursor: pointer;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.tab-close:hover {
  color: var(--danger-color);
  background: var(--border-color);
}

.tab-new {
  padding: 0 16px;
  font-size: 20px;
  color: var(--text-secondary);
  height: 100%;
  background: none;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-new:hover {
  color: var(--text-primary);
  background: var(--border-color);
}
</style>
