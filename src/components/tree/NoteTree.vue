<!-- src/components/tree/NoteTree.vue -->
<template>
  <div class="note-tree">
    <div class="tree-header">
      <span>笔记</span>
      <button @click="showNewFolder = true" title="新建文件夹">+</button>
    </div>
    <div class="tree-content">
      <TreeNode
        v-for="folder in notebookStore.folders"
        :key="folder.path"
        :folder="folder"
        :selected-path="notebookStore.currentFolder"
        @select="handleFolderSelect"
        @contextmenu="handleContext"
      />
    </div>
    <div class="tree-footer">
      <button class="footer-btn" @click="emit('openGraph')" title="图谱视图">🔗 图谱</button>
      <button class="footer-btn" @click="emit('openSearch')" title="搜索">🔍 搜索</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import TreeNode from './TreeNode.vue';

const emit = defineEmits<{
  openGraph: [];
  openSearch: [];
}>();

const notebookStore = useNotebookStore();
const showNewFolder = ref(false);

onMounted(() => { notebookStore.loadFolderTree(); });

async function handleFolderSelect(path: string) {
  await notebookStore.loadNotes(path);
}

function handleContext(_event: MouseEvent, _folder: any) {
  // Context menu for folder operations (new folder, rename, delete)
  // Will be implemented with a simple dropdown
}
</script>

<style scoped>
.note-tree {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  background: var(--bg-sidebar);
}
.tree-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; font-weight: 600; font-size: 14px;
  border-bottom: 1px solid var(--border-color);
}
.tree-header button { font-size: 18px; color: var(--text-secondary); padding: 0 4px; }
.tree-header button:hover { color: var(--text-primary); }
.tree-content { flex: 1; overflow-y: auto; padding: 4px 0; }
.tree-footer {
  display: flex; border-top: 1px solid var(--border-color); padding: 4px;
}
.footer-btn {
  flex: 1; padding: 6px; font-size: 12px; color: var(--text-secondary);
  text-align: center; border-radius: 4px;
}
.footer-btn:hover { background: var(--border-color); }
</style>
