<template>
  <div class="note-tree">
    <div class="tree-header">
      <span>笔记</span>
      <div class="header-actions">
        <button @click="handleNewFolder" title="新建文件夹">📁+</button>
      </div>
    </div>

    <div class="tree-content">
      <div v-if="notebookStore.folders.length === 0" class="empty-state">
        <p>暂无文件夹</p>
        <button class="create-first-btn" @click="handleNewFolder">创建第一个文件夹</button>
      </div>

      <TreeNode
        v-for="folder in notebookStore.folders"
        :key="folder.path"
        :folder="folder"
        :selected-path="notebookStore.currentFolder"
        :note-map="noteMap"
        @select="handleFolderSelect"
        @select-note="handleNoteClick"
        @contextmenu-folder="handleFolderContextMenu"
        @contextmenu-note="handleNoteContextMenu"
      />
    </div>

    <div class="tree-footer">
      <button class="footer-btn" @click="handleNewNote" title="新建笔记">
        <span class="btn-icon">📝</span>
        <span>新建笔记</span>
      </button>
      <button class="footer-btn" @click="emit('openGraph')" title="图谱视图">
        <span class="btn-icon">🔗</span>
        <span>图谱</span>
      </button>
      <button class="footer-btn" @click="emit('openSearch')" title="搜索">
        <span class="btn-icon">🔍</span>
        <span>搜索</span>
      </button>
    </div>

    <!-- Context Menu (simple implementation) -->
    <div
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    >
      <div v-if="contextMenu.type === 'folder'" class="menu-items">
        <button @click="handleMenuAction('new-note')">新建笔记</button>
        <button @click="handleMenuAction('new-subfolder')">新建子文件夹</button>
        <button @click="handleMenuAction('rename')">重命名</button>
        <button @click="handleMenuAction('delete')" class="danger">删除</button>
      </div>
      <div v-if="contextMenu.type === 'note'" class="menu-items">
        <button @click="handleMenuAction('rename-note')">重命名</button>
        <button @click="handleMenuAction('move-note')">移动</button>
        <button @click="handleMenuAction('delete-note')" class="danger">删除</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed, onUnmounted, ref } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import TreeNode from './TreeNode.vue';
import type { Folder, Note } from '@/types/notebook';

const emit = defineEmits<{
  openGraph: [];
  openSearch: [];
}>();

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  type: 'folder' as 'folder' | 'note',
  target: null as Folder | Note | null,
});

// Build a map of folder paths to notes
const noteMap = computed(() => {
  const map = new Map<string, Note[]>();
  notebookStore.notes.forEach(note => {
    const folder = note.folder;
    if (!map.has(folder)) {
      map.set(folder, []);
    }
    map.get(folder)!.push(note);
  });
  return map;
});

onMounted(async () => {
  await loadInitialData();
});

async function loadInitialData() {
  try {
    await notebookStore.loadFolderTree();
    // Load notes from default folder if available
    if (notebookStore.folders.length > 0) {
      const defaultFolder = notebookStore.folders[0].path;
      await notebookStore.loadNotes(defaultFolder);
    }
  } catch (error) {
    console.error('Failed to load initial data:', error);
  }
}

async function handleFolderSelect(path: string) {
  try {
    await notebookStore.loadNotes(path);
  } catch (error) {
    console.error('Failed to load folder:', error);
  }
}

async function handleNoteClick(note: Note) {
  try {
    await notebookStore.openNote(note.id);
    const currentNote = notebookStore.currentNote;
    if (currentNote) {
      editorStore.openTab(currentNote.note.id, currentNote.note.title);
    }
  } catch (error) {
    console.error('Failed to open note:', error);
  }
}

async function handleNewNote() {
  try {
    const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');
    const note = await notebookStore.createNote(folder, '新笔记', '');
    if (note) {
      await notebookStore.loadNotes(folder); // Reload notes
      editorStore.openTab(note.id, note.title);
      await notebookStore.openNote(note.id);
    }
  } catch (error) {
    console.error('Failed to create note:', error);
    alert('创建笔记失败: ' + (error as Error).message);
  }
}

async function handleNewFolder() {
  const name = prompt('请输入文件夹名称:');
  if (name && name.trim()) {
    try {
      const parent = notebookStore.currentFolder || '';
      await notebookStore.createFolder(parent, name.trim());
      await notebookStore.loadFolderTree();
    } catch (error) {
      console.error('Failed to create folder:', error);
      alert('创建文件夹失败: ' + (error as Error).message);
    }
  }
}

function handleFolderContextMenu(event: MouseEvent, folder: Folder) {
  event.preventDefault();
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    type: 'folder',
    target: folder,
  };
}

function handleNoteContextMenu(event: MouseEvent, note: Note) {
  event.preventDefault();
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    type: 'note',
    target: note,
  };
}

async function handleMenuAction(action: string) {
  contextMenu.value.show = false;
  const target = contextMenu.value.target;

  if (!target) return;

  try {
    switch (action) {
      case 'new-note':
        if (target instanceof Object && 'path' in target) {
          const folder = target as Folder;
          const title = prompt('笔记标题:', '新笔记');
          if (title) {
            const note = await notebookStore.createNote(folder.path, title, '');
            if (note) {
              await notebookStore.loadNotes(folder.path);
              editorStore.openTab(note.id, note.title);
              await notebookStore.openNote(note.id);
            }
          }
        }
        break;

      case 'new-subfolder':
        if (target instanceof Object && 'path' in target) {
          const folder = target as Folder;
          const name = prompt('子文件夹名称:');
          if (name) {
            await notebookStore.createFolder(folder.path, name);
            await notebookStore.loadFolderTree();
          }
        }
        break;

      case 'rename':
        if (target instanceof Object && 'name' in target && 'path' in target) {
          const folder = target as Folder;
          const newName = prompt('新名称:', folder.name);
          if (newName && newName !== folder.name) {
            await notebookStore.renameFolder(folder.path, newName);
            await notebookStore.loadFolderTree();
          }
        }
        break;

      case 'delete':
        if (target instanceof Object && 'name' in target && 'path' in target) {
          const folder = target as Folder;
          if (confirm(`确定要删除文件夹 "${folder.name}" 及其所有内容吗?`)) {
            await notebookStore.deleteFolder(folder.path);
            if (notebookStore.currentFolder === folder.path) {
              notebookStore.currentFolder = '';
              notebookStore.notes = [];
            }
            await notebookStore.loadFolderTree();
          }
        }
        break;

      case 'rename-note':
        if (target instanceof Object && 'id' in target && 'title' in target) {
          const note = target as Note;
          const newTitle = prompt('新标题:', note.title);
          // This would need a rename note function in the store
          // For now, we'll just log it
          console.log('Rename note:', note.id, 'to', newTitle);
        }
        break;

      case 'move-note':
        if (target instanceof Object && 'id' in target && 'title' in target) {
          // This would need a move note UI
          alert('移动功能需要实现文件夹选择器');
        }
        break;

      case 'delete-note':
        if (target instanceof Object && 'id' in target && 'title' in target) {
          const note = target as Note;
          if (confirm(`确定要删除笔记 "${note.title}" 吗?`)) {
            await notebookStore.deleteNote(note.id);
            editorStore.closeTab(note.id);
          }
        }
        break;
    }
  } catch (error) {
    console.error('Failed to perform action:', error);
    alert('操作失败: ' + (error as Error).message);
  }
}

// Close context menu when clicking outside
function handleGlobalClick() {
  if (contextMenu.value.show) {
    contextMenu.value.show = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleGlobalClick);
});

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick);
});
</script>

<style scoped>
.note-tree {
  width: var(--sidebar-width, 260px);
  min-width: var(--sidebar-width, 260px);
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border-color);
  background: var(--bg-sidebar);
  height: 100%;
}

.tree-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.header-actions button {
  font-size: 16px;
  color: var(--text-secondary);
  padding: 4px 8px;
  cursor: pointer;
  background: none;
  border: none;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.header-actions button:hover {
  color: var(--text-primary);
  background: var(--border-color);
}

.tree-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-secondary);
}

.empty-state p {
  margin-bottom: 16px;
  font-size: 14px;
}

.create-first-btn {
  padding: 8px 16px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  transition: opacity 0.15s ease;
}

.create-first-btn:hover {
  opacity: 0.9;
}

.tree-footer {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--border-color);
  padding: 8px;
  gap: 4px;
  background: var(--bg-secondary);
}

.footer-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px;
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
  border-radius: 6px;
  background: none;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.footer-btn:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

.btn-icon {
  font-size: 16px;
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 160px;
  padding: 4px;
}

.menu-items {
  display: flex;
  flex-direction: column;
}

.menu-items button {
  padding: 8px 12px;
  text-align: left;
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  transition: background 0.15s ease;
}

.menu-items button:hover {
  background: var(--border-color);
}

.menu-items button.danger {
  color: var(--danger-color);
}

.menu-items button.danger:hover {
  background: rgba(255, 77, 77, 0.1);
}
</style>
