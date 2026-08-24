// src/stores/notebook.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Note, Folder, NoteWithContent, SearchResult, UpdateNoteRequest } from '@/types/notebook';
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
  }

  async function loadAllNotes() {
    const allFolders = flattenFolders(folders.value);
    // Parallel loads: one list_notes per folder at once instead of serially.
    const results = await Promise.allSettled(
      allFolders.map(folder => notebookApi.listNotes(folder.path)),
    );
    const allNotes: Note[] = [];
    for (const r of results) {
      if (r.status === 'fulfilled') allNotes.push(...r.value);
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

  // --- pending-save flush -------------------------------------------------
  // NoteEditor registers a flusher so any note switch first persists the
  // previous note's debounced edits instead of silently dropping them.
  let pendingFlush: (() => Promise<void>) | null = null;

  function registerPendingSave(fn: (() => Promise<void>) | null) {
    pendingFlush = fn;
  }

  async function flushPendingSave() {
    const flush = pendingFlush;
    pendingFlush = null;
    if (flush) await flush();
  }

  // --- note opening with request sequencing -------------------------------
  let openSeq = 0;

  /** Open a note. Flushes the previous note's pending edits first and
   *  discards out-of-order responses from rapid switching. */
  async function openNote(id: string) {
    const seq = ++openSeq;
    await flushPendingSave();
    const result = await notebookApi.getNote(id);
    if (seq !== openSeq) return; // a newer openNote superseded this one
    currentNote.value = result;
  }

  async function createNote(folder: string, title: string, content: string, tags?: string[]) {
    const note = await notebookApi.createNote({ folder, title, content, tags });
    notes.value.unshift(note);
    await openNote(note.id);
    await loadFolderTree();
    return note;
  }

  /** Update content of a note by id, whether or not it is the open note. */
  async function updateNoteContent(id: string, content: string, title?: string) {
    const updated = await notebookApi.updateNote({ id, content, title });
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value = { note: updated, content };
    }
    return updated;
  }

  async function updateNoteTags(id: string, tags: string[]) {
    // Only send content when this is the open note (we have fresh content);
    // otherwise omit the field — the backend treats missing content as
    // "don't touch the file". Sending '' here used to wipe note files.
    const req: UpdateNoteRequest = { id, tags };
    if (currentNote.value?.note.id === id) {
      req.content = currentNote.value.content;
    }
    const updated = await notebookApi.updateNote(req);
    const idx = notes.value.findIndex(n => n.id === id);
    if (idx >= 0) notes.value[idx] = updated;
    if (currentNote.value?.note.id === id) {
      currentNote.value = { note: updated, content: currentNote.value.content };
    }
    return updated;
  }

  async function renameNote(id: string, newTitle: string) {
    // Same rule as updateNoteTags: never send content for a non-open note.
    const req: UpdateNoteRequest = { id, title: newTitle };
    if (currentNote.value?.note.id === id) {
      req.content = currentNote.value.content;
    }
    const updated = await notebookApi.updateNote(req);
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

  async function createFolder(parent: string, name: string) {
    const path = await notebookApi.createFolder(parent, name);
    await loadFolderTree();
    return path;
  }

  async function renameFolder(path: string, newName: string) {
    const newPath = await notebookApi.renameFolder(path, newName);
    await loadFolderTree();
    await loadAllNotes();
    return newPath;
  }

  async function deleteFolder(path: string) {
    await notebookApi.deleteFolder(path);
    await loadFolderTree();
    await loadAllNotes();
  }

  // --- search with response sequencing -------------------------------------
  let searchSeq = 0;

  async function search(query: string, scope?: string) {
    searchQuery.value = query;
    if (!query.trim()) {
      searchResults.value = [];
      return;
    }
    const seq = ++searchSeq;
    const results = await notebookApi.searchNotes(query, scope);
    if (seq !== searchSeq) return; // a newer search superseded this one
    searchResults.value = results;
  }

  return {
    folders, currentFolder, notes, currentNote, searchResults, searchQuery,
    viewMode, selectedFolderForList, folderNotes,
    currentNotes, loadFolderTree, loadNotes, loadAllNotes, openNote, createNote,
    updateNoteContent, updateNoteTags, deleteNote, search, createFolder, renameFolder, deleteFolder,
    renameNote, moveNote, registerPendingSave, flushPendingSave,
  };
});
