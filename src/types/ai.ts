// src/types/ai.ts
export interface AiOperation {
  id: string;
  note_id: string | null;
  operation_type: 'categorize' | 'rename' | 'enrich' | 'merge';
  before_state: string | null;
  after_state: string | null;
  status: 'pending' | 'applied' | 'rejected';
  created_at: string;
}

export interface AiSuggestion {
  action: 'create' | 'append' | 'rename' | 'tag' | 'optimize';
  title?: string;
  folder?: string;
  tags?: string[];
  content?: string;
  target_note_id?: string;
  confidence: number;
}

export interface FileAttachment {
  name: string;
  content: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  suggestions?: AiSuggestion[];
  status?: 'pending' | 'done' | 'error';
  reasoning?: string;
  attachments?: FileAttachment[];
}
