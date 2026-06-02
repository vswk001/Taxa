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

  const currentNotes = computed(() => notes.value);

  async function loadFolderTree() {
    folders.value = await notebookApi.getFolderTree();
  }

  async function loadNotes(folder: string) {
    currentFolder.value = folder;
    notes.value = await notebookApi.listNotes(folder);
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

  async function renameNote(id: string, newTitle: string) {
    const note = notes.value.find(n => n.id === id);
    if (!note) return;
    const updated = await notebookApi.updateNote({ id, title: newTitle, content: currentNote.value?.content || '' });
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value.note = updated;
    }
    return updated;
  }

  async function moveNote(id: string, targetFolder: string, newTitle?: string) {
    const updated = await notebookApi.moveNote({ id, target_folder: targetFolder, new_title: newTitle });
    // Remove from current list
    notes.value = notes.value.filter(n => n.id !== id);
    // Update current note if it's the one being moved
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

  async function search(query: string) {
    searchQuery.value = query;
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }
    searchResults.value = await notebookApi.searchNotes(query);
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
    currentNotes, loadFolderTree, loadNotes, openNote, createNote,
    updateNoteContent, deleteNote, search, createFolder, renameFolder, deleteFolder,
    renameNote, moveNote,
  };
});
