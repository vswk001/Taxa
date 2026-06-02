// src/types/ai-extended.ts
export interface OrganizeResult {
  action: string;
  title: string;
  folder: string;
  tags: string[];
  content: string;
  target_note_id: string | null;
  complexity: string;
}
