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
      <button class="tab-close" @click.stop="handleCloseTab(tab.id)">×</button>
    </div>
    <button class="tab-new" @click="handleNewNote" title="新建笔记">+</button>
  </div>
</template>

<script setup lang="ts">
import { useEditorStore } from '@/stores/editor';
import { useNotebookStore } from '@/stores/notebook';

const editorStore = useEditorStore();
const notebookStore = useNotebookStore();

async function handleTabClick(tabId: string) {
  editorStore.setActiveTab(tabId);
  // Load the note content when tab is clicked
  if (notebookStore.currentNote?.note.id !== tabId) {
    await notebookStore.openNote(tabId);
  }
}

async function handleNewNote() {
  const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');
  const note = await notebookStore.createNote(folder, '新笔记', '');
  if (note) {
    editorStore.openTab(note.id, note.title);
  }
}

function handleCloseTab(tabId: string) {
  editorStore.closeTab(tabId);
  // If closing the active tab, clear the current note
  if (notebookStore.currentNote?.note.id === tabId && editorStore.activeTabId !== tabId) {
    if (editorStore.activeTabId) {
      notebookStore.openNote(editorStore.activeTabId);
    } else {
      notebookStore.currentNote = null;
    }
  }
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: var(--tab-height, 36px);
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
}
.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  height: 100%;
  font-size: 13px;
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  color: var(--text-secondary);
  white-space: nowrap;
}
.tab:hover { background: var(--bg-primary); }
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
  font-size: 16px;
  color: var(--text-secondary);
  padding: 0 2px;
  background: none;
  border: none;
  cursor: pointer;
}
.tab-close:hover { color: var(--danger-color); }
.tab-new {
  padding: 0 12px;
  font-size: 18px;
  color: var(--text-secondary);
  height: 100%;
  background: none;
  border: none;
  cursor: pointer;
}
.tab-new:hover { color: var(--text-primary); }
</style>
