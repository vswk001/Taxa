<template>
  <div class="note-tree" @click="handleGlobalClick">
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
        :selected-note-id="notebookStore.currentNote?.note.id ?? null"
        :note-map="noteMap"
        @select="handleFolderSelect"
        @select-note="handleNoteClick"
        @contextmenu-folder="handleFolderContextMenu"
        @contextmenu-note="handleNoteContextMenu"
      />
    </div>

    <div class="tree-footer">
      <button class="footer-btn" @click="handleNewNote">
        <span class="btn-icon">📝</span><span>新建笔记</span>
      </button>
      <button class="footer-btn" @click="emit('openGraph')">
        <span class="btn-icon">🔗</span><span>图谱</span>
      </button>
      <button class="footer-btn" @click="emit('openSearch')">
        <span class="btn-icon">🔍</span><span>搜索</span>
      </button>
    </div>

    <div
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div v-if="contextMenu.type === 'folder'" class="menu-items">
        <button @click="handleMenuAction('new-note')">新建笔记</button>
        <button @click="handleMenuAction('new-subfolder')">新建子文件夹</button>
        <button @click="handleMenuAction('rename')">重命名</button>
        <button @click="handleMenuAction('delete')" class="danger">删除</button>
      </div>
      <div v-if="contextMenu.type === 'note'" class="menu-items">
        <button @click="handleMenuAction('rename-note')">重命名</button>
        <button @click="handleMenuAction('move-note')">移动到...</button>
        <button @click="handleMenuAction('delete-note')" class="danger">删除</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import TreeNode from './TreeNode.vue';
import type { Folder, Note } from '@/types/notebook';

const emit = defineEmits<{ openGraph: []; openSearch: [] }>();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

const contextMenu = ref({
  show: false, x: 0, y: 0,
  type: 'folder' as 'folder' | 'note',
  target: null as Folder | Note | null,
});

const noteMap = computed(() => {
  const map = new Map<string, Note[]>();
  notebookStore.notes.forEach(note => {
    const list = map.get(note.folder) || [];
    list.push(note);
    map.set(note.folder, list);
  });
  return map;
});

onMounted(async () => {
  await notebookStore.loadFolderTree();
  // Load ALL notes (not just from one folder) so the tree can display them
  await loadAllNotes();
});

async function loadAllNotes() {
  try {
    // Load notes for each folder
    for (const folder of notebookStore.folders) {
      await notebookStore.loadNotes(folder.path);
    }
  } catch (e) {
    console.error('Failed to load notes:', e);
  }
}

async function handleFolderSelect(path: string) {
  // Just highlight the folder, notes are already loaded
  notebookStore.currentFolder = path;
}

async function handleNoteClick(note: Note) {
  try {
    await notebookStore.openNote(note.id);
    if (notebookStore.currentNote) {
      editorStore.openTab(notebookStore.currentNote.note.id, notebookStore.currentNote.note.title);
    }
  } catch (e) {
    console.error('Failed to open note:', e);
  }
}

async function handleNewNote() {
  try {
    const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || '未分类');
    const title = prompt('笔记标题:', '新笔记');
    if (!title) return;
    const note = await notebookStore.createNote(folder, title, '');
    if (note) {
      editorStore.openTab(note.id, note.title);
      await loadAllNotes();
    }
  } catch (e: any) {
    console.error('Failed to create note:', e);
    alert('创建笔记失败: ' + (e.message || e));
  }
}

async function handleNewFolder() {
  const name = prompt('请输入文件夹名称:');
  if (name?.trim()) {
    try {
      await notebookStore.createFolder(notebookStore.currentFolder || '', name.trim());
      await notebookStore.loadFolderTree();
    } catch (e: any) {
      alert('创建文件夹失败: ' + (e.message || e));
    }
  }
}

function handleFolderContextMenu(event: MouseEvent, folder: Folder) {
  event.preventDefault();
  event.stopPropagation();
  contextMenu.value = { show: true, x: event.clientX, y: event.clientY, type: 'folder', target: folder };
}

function handleNoteContextMenu(event: MouseEvent, note: Note) {
  event.preventDefault();
  event.stopPropagation();
  contextMenu.value = { show: true, x: event.clientX, y: event.clientY, type: 'note', target: note };
}

async function handleMenuAction(action: string) {
  contextMenu.value.show = false;
  const target = contextMenu.value.target;
  if (!target) return;

  try {
    if (action === 'new-note' && 'path' in target) {
      const folder = target as Folder;
      const title = prompt('笔记标题:', '新笔记');
      if (title) {
        const note = await notebookStore.createNote(folder.path, title, '');
        if (note) {
          editorStore.openTab(note.id, note.title);
          await loadAllNotes();
        }
      }
    } else if (action === 'new-subfolder' && 'path' in target) {
      const folder = target as Folder;
      const name = prompt('子文件夹名称:');
      if (name) {
        await notebookStore.createFolder(folder.path, name);
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'rename' && 'path' in target && 'name' in target) {
      const folder = target as Folder;
      const newName = prompt('新名称:', folder.name);
      if (newName && newName !== folder.name) {
        await notebookStore.renameFolder(folder.path, newName);
        await notebookStore.loadFolderTree();
        await loadAllNotes();
      }
    } else if (action === 'delete' && 'path' in target && 'name' in target) {
      const folder = target as Folder;
      if (confirm(`确定要删除文件夹 "${folder.name}" 及其所有内容吗?`)) {
        await notebookStore.deleteFolder(folder.path);
        notebookStore.currentFolder = '';
        await notebookStore.loadFolderTree();
        await loadAllNotes();
      }
    } else if (action === 'rename-note' && 'id' in target) {
      const note = target as Note;
      const newTitle = prompt('新标题:', note.title);
      if (newTitle && newTitle !== note.title) {
        await notebookStore.renameNote(note.id, newTitle);
        await loadAllNotes();
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'move-note' && 'id' in target) {
      const note = target as Note;
      const folders = notebookStore.folders;
      const folderNames = folders.map(f => f.path).join('\n');
      const targetFolder = prompt(`移动到文件夹 (输入路径):\n${folderNames}`);
      if (targetFolder) {
        await notebookStore.moveNote(note.id, targetFolder);
        await loadAllNotes();
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'delete-note' && 'id' in target) {
      const note = target as Note;
      if (confirm(`确定要删除笔记 "${note.title}" 吗?`)) {
        await notebookStore.deleteNote(note.id);
        editorStore.closeTab(note.id);
      }
    }
  } catch (e: any) {
    console.error('Menu action failed:', e);
    alert('操作失败: ' + (e.message || e));
  }
}

function handleGlobalClick() {
  if (contextMenu.value.show) contextMenu.value.show = false;
}
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
  display: flex; justify-content: space-between; align-items: center;
  padding: 12px 16px; font-weight: 600; font-size: 14px;
  border-bottom: 1px solid var(--border-color); background: var(--bg-secondary);
}
.header-actions { display: flex; gap: 8px; }
.header-actions button {
  font-size: 16px; color: var(--text-secondary); padding: 4px 8px;
  cursor: pointer; background: none; border: none; border-radius: 4px;
}
.header-actions button:hover { color: var(--text-primary); background: var(--border-color); }

.tree-content { flex: 1; overflow-y: auto; padding: 8px 0; }

.empty-state {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; padding: 40px 20px; color: var(--text-secondary);
}
.empty-state p { margin-bottom: 16px; font-size: 14px; }
.create-first-btn {
  padding: 8px 16px; background: var(--accent-color); color: white;
  border: none; border-radius: 6px; cursor: pointer; font-size: 13px;
}
.create-first-btn:hover { opacity: 0.9; }

.tree-footer {
  display: flex; flex-direction: column;
  border-top: 1px solid var(--border-color); padding: 8px; gap: 4px;
  background: var(--bg-secondary);
}
.footer-btn {
  display: flex; align-items: center; justify-content: center; gap: 8px;
  padding: 10px; font-size: 13px; color: var(--text-secondary);
  text-align: center; border-radius: 6px; background: none; border: none; cursor: pointer;
}
.footer-btn:hover { background: var(--border-color); color: var(--text-primary); }
.btn-icon { font-size: 16px; }

.context-menu {
  position: fixed; background: var(--bg-primary);
  border: 1px solid var(--border-color); border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 1000;
  min-width: 160px; padding: 4px;
}
.menu-items { display: flex; flex-direction: column; }
.menu-items button {
  padding: 8px 12px; text-align: left; background: none; border: none;
  border-radius: 4px; cursor: pointer; font-size: 13px; color: var(--text-primary);
}
.menu-items button:hover { background: var(--border-color); }
.menu-items button.danger { color: var(--danger-color); }
.menu-items button.danger:hover { background: rgba(255,77,77,0.1); }
</style>
