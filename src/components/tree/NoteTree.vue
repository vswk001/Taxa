<template>
  <div class="note-tree" @click="handleGlobalClick">
    <div class="tree-header">
      <span>笔记</span>
      <div class="header-actions">
        <button @click="handleNewNote" title="新建笔记">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="12" y1="18" x2="12" y2="12"/><line x1="9" y1="15" x2="15" y2="15"/></svg>
        </button>
        <button @click="handleNewFolder" title="新建文件夹">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>
        </button>
        <button @click="toggleExpandAll" :title="isAllExpanded ? '全部收缩' : '全部展开'">
          <svg v-if="isAllExpanded" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 14 10 14 10 20"/><polyline points="20 10 14 10 14 4"/><line x1="14" y1="10" x2="21" y2="3"/><line x1="3" y1="21" x2="10" y2="14"/></svg>
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/></svg>
        </button>
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
        :expand-version="expandVersion"
        :expand-target="expandTarget"
        @select="handleFolderSelect"
        @select-note="handleNoteClick"
        @contextmenu-folder="handleFolderContextMenu"
        @contextmenu-note="handleNoteContextMenu"
      />
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
        <div class="menu-separator"></div>
        <button @click="handleMenuAction('import-file')">导入文件...</button>
        <button @click="handleMenuAction('import-folder')">导入目录...</button>
        <button @click="handleMenuAction('export-folder')">导出目录...</button>
        <div class="menu-separator"></div>
        <button @click="handleMenuAction('rename')">重命名</button>
        <button @click="handleMenuAction('delete')" class="danger">删除</button>
      </div>
      <div v-if="contextMenu.type === 'note'" class="menu-items">
        <button @click="handleMenuAction('rename-note')">重命名</button>
        <button @click="handleMenuAction('move-note')">移动到...</button>
        <button @click="handleMenuAction('export-note')">导出为文件...</button>
        <div class="menu-separator"></div>
        <button @click="handleMenuAction('delete-note')" class="danger">删除</button>
      </div>
    </div>

    <InputDialog
      :visible="inputDialog.show"
      :title="inputDialog.title"
      :placeholder="inputDialog.placeholder"
      :default-value="inputDialog.defaultValue"
      @confirm="inputDialog.onConfirm"
      @cancel="inputDialog.show = false"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import { invoke } from '@tauri-apps/api/core';
import { confirm as tauriConfirm, message as tauriMessage } from '@tauri-apps/plugin-dialog';
import TreeNode from './TreeNode.vue';
import InputDialog from '@/components/common/InputDialog.vue';
import type { Folder, Note } from '@/types/notebook';

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

const expandVersion = ref(0);
const expandTarget = ref(false);
const isAllExpanded = ref(false);

function toggleExpandAll() {
  isAllExpanded.value = !isAllExpanded.value;
  expandTarget.value = isAllExpanded.value;
  expandVersion.value++;
}

const contextMenu = ref({
  show: false, x: 0, y: 0,
  type: 'folder' as 'folder' | 'note',
  target: null as Folder | Note | null,
});

const inputDialog = ref<{
  show: boolean;
  title: string;
  placeholder: string;
  defaultValue: string;
  onConfirm: (value: string) => void;
}>({
  show: false, title: '', placeholder: '', defaultValue: '',
  onConfirm: () => {},
});

function showInputDialog(opts: {
  title: string;
  placeholder?: string;
  defaultValue?: string;
}): Promise<string | null> {
  return new Promise((resolve) => {
    inputDialog.value = {
      show: true,
      title: opts.title,
      placeholder: opts.placeholder || '',
      defaultValue: opts.defaultValue || '',
      onConfirm: (value: string) => {
        inputDialog.value.show = false;
        resolve(value || null);
      },
    };
  });
}

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
  await loadAllNotes();
});

async function loadAllNotes() {
  try {
    for (const folder of notebookStore.folders) {
      await notebookStore.loadNotes(folder.path);
    }
  } catch (e) {
    console.error('Failed to load notes:', e);
  }
}

async function handleFolderSelect(path: string) {
  notebookStore.currentFolder = path;
  notebookStore.currentNote = null;
  notebookStore.viewMode = 'folder';
  notebookStore.selectedFolderForList = path;
}

async function handleNoteClick(note: Note) {
  try {
    notebookStore.currentFolder = '';
    notebookStore.viewMode = 'editor';
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
    const title = await showInputDialog({ title: '新建笔记', placeholder: '笔记标题', defaultValue: '新笔记' });
    if (!title) return;
    const note = await notebookStore.createNote(folder, title, '');
    if (note) {
      editorStore.openTab(note.id, note.title);
      await loadAllNotes();
    }
  } catch (e: any) {
    console.error('Failed to create note:', e);
    await tauriMessage(e.message || String(e), { title: '创建笔记失败', kind: 'error' });
  }
}

async function handleNewFolder() {
  const name = await showInputDialog({ title: '新建文件夹', placeholder: '文件夹名称' });
  if (name?.trim()) {
    try {
      await notebookStore.createFolder(notebookStore.currentFolder || '', name.trim());
      await notebookStore.loadFolderTree();
    } catch (e: any) {
      await tauriMessage(e.message || String(e), { title: '创建文件夹失败', kind: 'error' });
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
      const title = await showInputDialog({ title: '新建笔记', placeholder: '笔记标题', defaultValue: '新笔记' });
      if (title) {
        const note = await notebookStore.createNote(folder.path, title, '');
        if (note) {
          editorStore.openTab(note.id, note.title);
          await loadAllNotes();
        }
      }
    } else if (action === 'new-subfolder' && 'path' in target) {
      const folder = target as Folder;
      const name = await showInputDialog({ title: '新建子文件夹', placeholder: '子文件夹名称' });
      if (name) {
        await notebookStore.createFolder(folder.path, name);
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'rename' && 'path' in target && 'name' in target) {
      const folder = target as Folder;
      const newName = await showInputDialog({ title: '重命名文件夹', placeholder: '新名称', defaultValue: folder.name });
      if (newName && newName !== folder.name) {
        await notebookStore.renameFolder(folder.path, newName);
        await notebookStore.loadFolderTree();
        await loadAllNotes();
      }
    } else if (action === 'delete' && 'path' in target && 'name' in target) {
      const folder = target as Folder;
      const yes = await tauriConfirm(`确定要删除文件夹 "${folder.name}" 及其所有内容吗?`, { title: '删除确认', kind: 'warning' });
      if (yes) {
        await notebookStore.deleteFolder(folder.path);
        notebookStore.currentFolder = '';
        await notebookStore.loadFolderTree();
        await loadAllNotes();
      }
    } else if (action === 'rename-note' && 'id' in target) {
      const note = target as Note;
      const newTitle = await showInputDialog({ title: '重命名笔记', placeholder: '新标题', defaultValue: note.title });
      if (newTitle && newTitle !== note.title) {
        await notebookStore.renameNote(note.id, newTitle);
        await loadAllNotes();
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'move-note' && 'id' in target) {
      const note = target as Note;
      const folders = notebookStore.folders;
      const folderPaths = folders.map(f => f.path).join(', ');
      const targetFolder = await showInputDialog({ title: '移动笔记', placeholder: folderPaths });
      if (targetFolder) {
        await notebookStore.moveNote(note.id, targetFolder);
        await loadAllNotes();
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'delete-note' && 'id' in target) {
      const note = target as Note;
      const yes = await tauriConfirm(`确定要删除笔记 "${note.title}" 吗?`, { title: '删除确认', kind: 'warning' });
      if (yes) {
        await notebookStore.deleteNote(note.id);
        editorStore.closeTab(note.id);
      }
    } else if (action === 'import-file' && 'path' in target) {
      const folder = target as Folder;
      const result = await invoke<{ title: string; content: string } | null>('import_note');
      if (result) {
        const note = await notebookStore.createNote(folder.path, result.title, result.content);
        if (note) { editorStore.openTab(note.id, note.title); await loadAllNotes(); }
      }
    } else if (action === 'import-folder' && 'path' in target) {
      const folder = target as Folder;
      const result = await invoke<{ folder: string; notes: { title: string; content: string }[] } | null>('import_folder');
      if (result) {
        for (const n of result.notes) {
          await notebookStore.createNote(folder.path, n.title, n.content);
        }
        await loadAllNotes();
        await notebookStore.loadFolderTree();
      }
    } else if (action === 'export-folder' && 'path' in target) {
      const folder = target as Folder;
      await invoke('export_folder', { folder: folder.path });
    } else if (action === 'export-note' && 'id' in target) {
      const note = target as Note;
      await notebookStore.openNote(note.id);
      const current = notebookStore.currentNote;
      if (current) {
        await invoke('export_note', { title: current.note.title, content: current.content });
      }
    }
  } catch (e: any) {
    console.error('Menu action failed:', e);
    await tauriMessage(e.message || String(e), { title: '操作失败', kind: 'error' });
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
.header-actions { display: flex; gap: 2px; }
.header-actions button {
  display: flex; align-items: center; justify-content: center;
  color: var(--text-secondary); padding: 3px 5px;
  cursor: pointer; background: none; border: none; border-radius: 3px;
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

.context-menu {
  position: fixed; background: var(--bg-primary);
  border: 1px solid var(--border-color); border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 1000;
  min-width: 160px; padding: 4px;
}
.menu-items { display: flex; flex-direction: column; }
.menu-items .menu-separator { height: 1px; background: var(--border-color); margin: 4px 0; }
.menu-items button {
  padding: 8px 12px; text-align: left; background: none; border: none;
  border-radius: 4px; cursor: pointer; font-size: 13px; color: var(--text-primary);
}
.menu-items button:hover { background: var(--border-color); }
.menu-items button.danger { color: var(--danger-color); }
.menu-items button.danger:hover { background: rgba(255,77,77,0.1); }
</style>
