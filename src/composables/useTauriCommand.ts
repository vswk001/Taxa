// src/composables/useTauriCommand.ts
import { invoke } from '@tauri-apps/api/core';

export async function tauriCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

// Notebook commands
export const notebookApi = {
  createNote: (req: import('@/types/notebook').CreateNoteRequest) =>
    tauriCommand<import('@/types/notebook').Note>('create_note', { req }),
  getNote: (id: string) =>
    tauriCommand<import('@/types/notebook').NoteWithContent>('get_note', { id }),
  updateNote: (req: import('@/types/notebook').UpdateNoteRequest) =>
    tauriCommand<import('@/types/notebook').Note>('update_note', { req }),
  deleteNote: (id: string) =>
    tauriCommand<void>('delete_note', { id }),
  moveNote: (req: import('@/types/notebook').MoveNoteRequest) =>
    tauriCommand<import('@/types/notebook').Note>('move_note', { req }),
  listNotes: (folder: string) =>
    tauriCommand<import('@/types/notebook').Note[]>('list_notes', { folder }),
  getFolderTree: () =>
    tauriCommand<import('@/types/notebook').Folder[]>('get_folder_tree'),
  searchNotes: (query: string) =>
    tauriCommand<import('@/types/notebook').SearchResult[]>('search_notes', { query }),
  createFolder: (parent: string, name: string) =>
    tauriCommand<string>('create_folder', { parent, name }),
  renameFolder: (path: string, newName: string) =>
    tauriCommand<string>('rename_folder', { path, newName }),
  deleteFolder: (path: string) =>
    tauriCommand<void>('delete_folder', { path }),
};
