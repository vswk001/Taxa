// src/stores/notebook.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Note, Folder, NoteWithContent, SearchResult } from '@/types/notebook';
import { notebookApi } from '@/composables/useTauriCommand';

export const useNotebookStore = defineStore('notebook', () => {
  const folders = ref<Folder[]>([]);
  const currentFolder = ref<string>('');
  const notes = ref<Note[]>([]);
  const currentNote = ref<NoteWithContent | null>(null);
  const searchResults = ref<SearchResult[]>([]);
  const searchQuery = ref('');
  const viewMode = ref<'editor' | 'folder'>('editor');
  const selectedFolderForList = ref('');

  const currentNotes = computed(() => notes.value);

  const folderNotes = computed(() => {
    const folder = selectedFolderForList.value;
    if (!folder) return [];
    return notes.value
      .filter(n => n.folder === folder || n.folder.startsWith(folder + '/'))
      .sort((a, b) => b.updated_at.localeCompare(a.updated_at));
  });

  async function loadFolderTree() {
    folders.value = await notebookApi.getFolderTree();
  }

  async function loadNotes(folder: string) {
    const folderNotes = await notebookApi.listNotes(folder);
    // Merge: add new notes, update existing, don't remove notes from other folders
    for (const note of folderNotes) {
      const idx = notes.value.findIndex(n => n.id === note.id);
      if (idx >= 0) {
        notes.value[idx] = note;
      } else {
        notes.value.push(note);
      }
    }
    currentFolder.value = folder;
  }

  async function loadAllNotes() {
    const allNotes: Note[] = [];
    const allFolders = flattenFolders(folders.value);
    for (const folder of allFolders) {
      try {
        const folderNotes = await notebookApi.listNotes(folder.path);
        allNotes.push(...folderNotes);
      } catch (e) {
        console.error(`Failed to load notes from ${folder.path}:`, e);
      }
    }
    notes.value = allNotes;
  }

  function flattenFolders(list: Folder[]): Folder[] {
    const result: Folder[] = [];
    for (const f of list) {
      result.push(f);
      if (f.children.length) result.push(...flattenFolders(f.children));
    }
    return result;
  }

  async function openNote(id: string) {
    currentNote.value = await notebookApi.getNote(id);
  }

  async function createNote(folder: string, title: string, content: string, tags?: string[]) {
    const note = await notebookApi.createNote({ folder, title, content, tags });
    notes.value.unshift(note);
    await openNote(note.id);
    await loadFolderTree();
    return note;
  }

  async function updateNoteContent(id: string, content: string, title?: string) {
    if (!currentNote.value) return;
    const updated = await notebookApi.updateNote({ id, content, title });
    currentNote.value = { note: updated, content };
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    return updated;
  }

  async function updateNoteTags(id: string, tags: string[]) {
    const content = currentNote.value?.note.id === id ? currentNote.value.content : '';
    const updated = await notebookApi.updateNote({ id, content, tags });
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value = { note: updated, content };
    }
    return updated;
  }

  async function renameNote(id: string, newTitle: string) {
    const content = currentNote.value?.note.id === id ? currentNote.value.content : '';
    const updated = await notebookApi.updateNote({ id, title: newTitle, content });
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value.note = updated;
    }
    return updated;
  }

  async function moveNote(id: string, targetFolder: string, newTitle?: string) {
    const updated = await notebookApi.moveNote({ id, target_folder: targetFolder, new_title: newTitle });
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value.note = updated;
    }
    return updated;
  }

  async function deleteNote(id: string) {
    await notebookApi.deleteNote(id);
    notes.value = notes.value.filter(n => n.id !== id);
    if (currentNote.value?.note.id === id) {
      currentNote.value = null;
    }
    await loadFolderTree();
  }

  async function search(query: string, scope?: string) {
    searchQuery.value = query;
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }
    searchResults.value = await notebookApi.searchNotes(query, scope);
  }

  async function createFolder(parent: string, name: string) {
    return await notebookApi.createFolder(parent, name);
  }

  async function renameFolder(path: string, newName: string) {
    return await notebookApi.renameFolder(path, newName);
  }

  async function deleteFolder(path: string) {
    return await notebookApi.deleteFolder(path);
  }

  return {
    folders, currentFolder, notes, currentNote, searchResults, searchQuery,
    viewMode, selectedFolderForList, folderNotes,
    currentNotes, loadFolderTree, loadNotes, loadAllNotes, openNote, createNote,
    updateNoteContent, updateNoteTags, deleteNote, search, createFolder, renameFolder, deleteFolder,
    renameNote, moveNote,
  };
});
