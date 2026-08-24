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
