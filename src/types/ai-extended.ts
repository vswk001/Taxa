// src/types/ai-extended.ts
export type OrganizeAction = 'create' | 'append';
export type Complexity = 'simple' | 'complex';

export interface OrganizeResult {
  action: OrganizeAction;
  title: string;
  folder: string;
  tags: string[];
  content: string;
  target_note_id: string | null;
  complexity: Complexity;
  reasoning?: string;
}

export interface LibrarySuggestion {
  type: string; // serde rename in Rust sends "type"
  note_id: string;
  target_folder?: string | null;
  merge_with_id?: string | null;
  tags: string[];
  reason: string;
}

export interface LibraryItem {
  suggestion: LibrarySuggestion;
  status: 'pending' | 'applying' | 'done' | 'error';
}
