<template>
  <div class="note-tree">
    <div class="tree-header">
      <span>笔记</span>
      <button @click="handleNewFolder" title="新建文件夹">+</button>
    </div>
    <div class="tree-content">
      <TreeNode
        v-for="folder in notebookStore.folders"
        :key="folder.path"
        :folder="folder"
        :selected-path="notebookStore.currentFolder"
        @select="handleFolderSelect"
        @contextmenu="handleContextMenu"
      />
      <div v-if="notebookStore.currentNotes.length > 0" class="notes-list">
        <div
          v-for="note in notebookStore.currentNotes"
          :key="note.id"
          class="note-item"
          :class="{ active: notebookStore.currentNote?.note.id === note.id }"
          @click="handleNoteClick(note.id)"
        >
          <span class="note-icon">📄</span>
          <span class="note-title">{{ note.title }}</span>
          <button class="note-delete" @click.stop="handleDeleteNote(note.id)" title="删除">×</button>
        </div>
      </div>
    </div>
    <div class="tree-footer">
      <button class="footer-btn" @click="handleNewNote" title="新建笔记">+ 新建笔记</button>
      <button class="footer-btn" @click="emit('openGraph')" title="图谱视图">🔗 图谱</button>
      <button class="footer-btn" @click="emit('openSearch')" title="搜索">🔍 搜索</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import TreeNode from './TreeNode.vue';

const emit = defineEmits<{
  openGraph: [];
  openSearch: [];
}>();

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

onMounted(() => {
  notebookStore.loadFolderTree();
  // Load notes from default folder if available
  if (notebookStore.folders.length > 0) {
    const defaultFolder = notebookStore.folders[0].path;
    notebookStore.loadNotes(defaultFolder);
  }
});

async function handleFolderSelect(path: string) {
  await notebookStore.loadNotes(path);
}

async function handleNoteClick(noteId: string) {
  await notebookStore.openNote(noteId);
  const note = notebookStore.currentNote;
  if (note) {
    editorStore.openTab(note.note.id, note.note.title);
  }
}

async function handleNewNote() {
  const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');
  await notebookStore.createNote(folder, '新笔记', '');
  // Open the newly created note in editor
  if (notebookStore.currentNote) {
    editorStore.openTab(notebookStore.currentNote.note.id, notebookStore.currentNote.note.title);
  }
}

async function handleDeleteNote(noteId: string) {
  if (confirm('确定要删除这个笔记吗?')) {
    await notebookStore.deleteNote(noteId);
    editorStore.closeTab(noteId);
  }
}

function handleNewFolder() {
  const name = prompt('请输入文件夹名称:');
  if (name) {
    const parent = notebookStore.currentFolder || '';
    notebookStore.createFolder?.(parent, name).then(() => {
      notebookStore.loadFolderTree();
    });
  }
}

function handleContextMenu(event: MouseEvent, folder: any) {
  event.preventDefault();
  // Simple context menu implementation
  const action = prompt(
    `文件夹操作:\n1. 重命名\n2. 删除\n\n输入数字选择:`,
    '1'
  );
  if (action === '1') {
    const newName = prompt('新名称:', folder.name);
    if (newName && newName !== folder.name) {
      notebookStore.renameFolder?.(folder.path, newName).then(() => {
        notebookStore.loadFolderTree();
      });
    }
  } else if (action === '2') {
    if (confirm(`确定要删除文件夹 "${folder.name}" 吗?`)) {
      notebookStore.deleteFolder?.(folder.path).then(() => {
        notebookStore.loadFolderTree();
        if (notebookStore.currentFolder === folder.path) {
          notebookStore.currentFolder = '';
          notebookStore.notes = [];
        }
      });
    }
  }
}
</script>

<style scoped>
.note-tree {
  width: var(--sidebar-width, 240px);
  min-width: var(--sidebar-width, 240px);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  background: var(--bg-sidebar);
}
.tree-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid var(--border-color);
}
.tree-header button {
  font-size: 18px;
  color: var(--text-secondary);
  padding: 0 4px;
  cursor: pointer;
  background: none;
  border: none;
}
.tree-header button:hover { color: var(--text-primary); }
.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}
.notes-list {
  padding: 4px 0;
}
.note-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
}
.note-item:hover {
  background: var(--border-color);
}
.note-item.active {
  background: var(--accent-color);
  color: white;
}
.note-icon {
  font-size: 14px;
}
.note-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.note-delete {
  font-size: 16px;
  color: var(--text-secondary);
  padding: 0 2px;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0;
}
.note-item:hover .note-delete {
  opacity: 1;
}
.note-delete:hover {
  color: var(--danger-color);
}
.tree-footer {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--border-color);
  padding: 4px;
  gap: 2px;
}
.footer-btn {
  flex: 1;
  padding: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
  border-radius: 4px;
  background: none;
  border: none;
  cursor: pointer;
}
.footer-btn:hover {
  background: var(--border-color);
}
</style>
