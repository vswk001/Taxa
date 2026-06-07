<template>
  <div class="app-layout">
    <div class="main-content">
      <!-- Left activity bar -->
      <div class="activity-bar left-activity">
        <button
          class="activity-icon"
          :class="{ active: sidebarVisible }"
          @click="sidebarVisible = !sidebarVisible"
          title="资源管理器 (Ctrl+B)"
        >📂</button>
        <button class="activity-icon" @click="showSearch = !showSearch" title="搜索 (Ctrl+K)">🔍</button>
        <button class="activity-icon" @click="openGraphTab" title="图谱 (Ctrl+G)">🔗</button>
      </div>

      <!-- Left sidebar -->
      <NoteTree v-if="sidebarVisible" />

      <div class="center-panel">
        <TabBar />
        <div class="editor-container">
          <GraphView v-if="editorStore.activeTabId === '__graph__'" />
          <template v-else>
            <FolderFileList v-if="notebookStore.viewMode === 'folder'" />
            <NoteEditor v-else />
          </template>
        </div>
      </div>

      <!-- Right sidebar -->
      <AiSidebar v-if="aiSidebarVisible" />
      <div class="activity-bar right-activity">
        <button
          class="activity-icon"
          :class="{ active: aiSidebarVisible }"
          @click="aiSidebarVisible = !aiSidebarVisible"
          title="AI 助手"
        >🤖</button>
      </div>
    </div>
    <SearchPanel v-if="showSearch" :visible="showSearch" @close="showSearch = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import NoteTree from '@/components/tree/NoteTree.vue';
import NoteEditor from '@/components/editor/NoteEditor.vue';
import FolderFileList from '@/components/folder/FolderFileList.vue';
import TabBar from '@/components/editor/TabBar.vue';
import AiSidebar from '@/components/ai/AiSidebar.vue';
import SearchPanel from '@/components/search/SearchPanel.vue';
import GraphView from '@/components/graph/GraphView.vue';

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const showSearch = ref(false);
const sidebarVisible = ref(true);
const aiSidebarVisible = ref(true);

function openGraphTab() {
  editorStore.openTab('__graph__', '笔记图谱');
}

function handleKeyboard(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;
  if (ctrl && e.key === 'k') { e.preventDefault(); showSearch.value = !showSearch.value; }
  if (ctrl && e.key === 'g') { e.preventDefault(); openGraphTab(); }
  if (ctrl && e.key === 'b') { e.preventDefault(); sidebarVisible.value = !sidebarVisible.value; }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyboard);
  notebookStore.loadFolderTree();
});
onUnmounted(() => document.removeEventListener('keydown', handleKeyboard));
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}
.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.activity-bar {
  width: 40px;
  min-width: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 8px;
  gap: 4px;
  background: var(--bg-secondary);
  user-select: none;
}
.left-activity {
  border-right: 1px solid var(--border-color);
}
.right-activity {
  border-left: 1px solid var(--border-color);
}
.activity-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  opacity: 0.6;
  transition: opacity 0.15s, background 0.15s;
}
.activity-icon:hover {
  opacity: 1;
  background: var(--border-color);
}
.activity-icon.active {
  opacity: 1;
  background: var(--border-color);
  border-left: 2px solid var(--accent-color);
}
.right-activity .activity-icon.active {
  border-left: none;
  border-right: 2px solid var(--accent-color);
}
.center-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}
.editor-container {
  flex: 1;
  overflow: hidden;
  position: relative;
}
</style>
