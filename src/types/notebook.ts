// src/types/notebook.ts
export interface Note {
  id: string;
  path: string;
  title: string;
  folder: string;
  tags: string[];
  created_at: string;
  updated_at: string;
  word_count: number;
  summary: string | null;
  ai_categorized: boolean;
}

export interface Folder {
  name: string;
  path: string;
  children: Folder[];
  note_count: number;
}

export interface NoteWithContent {
  note: Note;
  content: string;
}

export interface CreateNoteRequest {
  folder: string;
  title: string;
  content: string;
  tags?: string[];
}

export interface UpdateNoteRequest {
  id: string;
  title?: string;
  content?: string;
  folder?: string;
  tags?: string[];
}

export interface MoveNoteRequest {
  id: string;
  target_folder: string;
  new_title?: string;
}

export interface SearchResult {
  id: string;
  title: string;
  path: string;
  snippet: string;
  rank: number;
}
