<template>
  <div class="app-layout">
    <MenuBar />
    <div class="main-content">
      <NoteTree @openGraph="showGraph = true" @openSearch="showSearch = true" />
      <div class="center-panel">
        <TabBar />
        <div class="editor-area">
          <NoteEditor v-if="notebookStore.currentNote" />
          <div v-else class="empty-state">
            <p>选择或创建一个笔记开始</p>
          </div>
        </div>
      </div>
      <AiSidebar />
    </div>
    <StatusBar />
    <SearchPanel :visible="showSearch" @close="showSearch = false" />
    <GraphView :visible="showGraph" @close="showGraph = false" />
    <SettingsDialog :visible="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import MenuBar from './MenuBar.vue';
import StatusBar from './StatusBar.vue';
import NoteTree from '@/components/tree/NoteTree.vue';
import NoteEditor from '@/components/editor/NoteEditor.vue';
import TabBar from '@/components/editor/TabBar.vue';
import AiSidebar from '@/components/ai/AiSidebar.vue';
import SearchPanel from '@/components/search/SearchPanel.vue';
import GraphView from '@/components/graph/GraphView.vue';
import SettingsDialog from '@/components/settings/SettingsDialog.vue';

const notebookStore = useNotebookStore();
const showSearch = ref(false);
const showGraph = ref(false);
const showSettings = ref(false);

function handleKeyboard(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;
  if (ctrl && e.key === 'k') { e.preventDefault(); showSearch.value = !showSearch.value; }
  if (ctrl && e.key === 'g') { e.preventDefault(); showGraph.value = !showGraph.value; }
  if (ctrl && e.key === ',') { e.preventDefault(); showSettings.value = !showSettings.value; }
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
.center-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.editor-area {
  flex: 1;
  overflow: auto;
}
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
  font-size: 16px;
}
</style>
