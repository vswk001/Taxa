<template>
  <div class="app-layout">
    <div class="main-content">
      <!-- Left activity bar -->
      <div class="activity-bar left-activity">
        <div class="activity-top">
          <button
            class="activity-icon"
            :class="{ active: sidebarVisible }"
            @click="sidebarVisible = !sidebarVisible"
            :title="t('tree.notes') + ' (Ctrl+B)'"
          >📂</button>
          <button class="activity-icon" @click="showSearch = !showSearch" :title="t('search.placeholder') + ' (Ctrl+K)'">🔍</button>
          <button class="activity-icon" @click="openGraphTab" :title="t('graph.title') + ' (Ctrl+G)'">🔗</button>
        </div>
        <div class="activity-bottom">
          <button class="activity-icon" @click="showSettings = true" :title="t('settings.title')">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
          </button>
        </div>
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
          :title="t('ai.assistant')"
        >🤖</button>
      </div>
    </div>
    <SearchPanel v-if="showSearch" :visible="showSearch" @close="showSearch = false" />
    <SettingsDialog v-if="showSettings" :visible="showSettings" @close="showSettings = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import NoteTree from '@/components/tree/NoteTree.vue';
import NoteEditor from '@/components/editor/NoteEditor.vue';
import FolderFileList from '@/components/folder/FolderFileList.vue';
import TabBar from '@/components/editor/TabBar.vue';
import AiSidebar from '@/components/ai/AiSidebar.vue';
import SearchPanel from '@/components/search/SearchPanel.vue';
import GraphView from '@/components/graph/GraphView.vue';
import SettingsDialog from '@/components/settings/SettingsDialog.vue';

const { t } = useI18n();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const showSearch = ref(false);
const showSettings = ref(false);
const sidebarVisible = ref(true);
const aiSidebarVisible = ref(true);

function openGraphTab() {
  editorStore.openTab('__graph__', t('graph.title'));
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
  background: var(--bg-secondary);
  user-select: none;
}
.activity-top {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 8px;
  gap: 4px;
}
.activity-bottom {
  padding: 8px 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  border-top: 1px solid var(--border-color);
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
