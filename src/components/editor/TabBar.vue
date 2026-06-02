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

async function handleTabClick(tabId: string) {
  // Save current note before switching
  if (notebookStore.currentNote && notebookStore.currentNote.note.id !== tabId) {
    // The editor component will handle saving on blur/change
  }

  // Set the active tab
  editorStore.setActiveTab(tabId);

  // Load the note content if it's not already loaded
  if (notebookStore.currentNote?.note.id !== tabId) {
    try {
      await notebookStore.openNote(tabId);
    } catch (error) {
      console.error('Failed to load note:', error);
    }
  }
}

async function handleNewNote() {
  try {
    // Get the current folder or use default
    const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');

    // Create a new note
    const note = await notebookStore.createNote(folder, '新笔记', '');

    if (note) {
      // Open it in the editor
      editorStore.openTab(note.id, note.title);
      await notebookStore.openNote(note.id);
    }
  } catch (error) {
    console.error('Failed to create new note:', error);
  }
}

async function handleCloseTab(tabId: string) {
  // Close the tab in the editor store
  editorStore.closeTab(tabId);

  // If we closed the current note, clear it or switch to another
  if (notebookStore.currentNote?.note.id === tabId) {
    // Check if there's a new active tab
    if (editorStore.activeTabId) {
      await notebookStore.openNote(editorStore.activeTabId);
    } else {
      // No more tabs, clear current note
      notebookStore.currentNote = null;
    }
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
