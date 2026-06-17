// src/stores/ai.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ChatMessage, FileAttachment } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import i18n from '@/i18n';
import { useNotebookStore } from './notebook';

/** Translate via the global i18n instance — works inside/outside setup. */
function t(key: string, named?: Record<string, unknown>): string {
  return i18n.global.t(key, named as any);
}

function extractError(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  if (e && typeof e === 'object') {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === 'string') return obj.message;
    if (typeof obj.cause === 'string') return obj.cause;
    try { return JSON.stringify(e); } catch { /* fall through */ }
  }
  return t('common.unknownError');
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(t('ai.requestTimeout', { n: ms / 1000 }))), ms);
    promise.then(
      (val) => { clearTimeout(timer); resolve(val); },
      (err) => { clearTimeout(timer); reject(err); },
    );
  });
}

export const useAiStore = defineStore('ai', () => {
  const messages = ref<ChatMessage[]>([]);
  const isProcessing = ref(false);
  const lastResult = ref<OrganizeResult | null>(null);
  const mode = ref<'organize' | 'optimize'>('organize');
  let requestSeq = 0;
  let streamUnlisten: UnlistenFn | null = null;

  async function submitInput(content: string, attachments?: FileAttachment[]) {
    const seq = ++requestSeq;

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
      status: 'done',
      attachments: attachments?.length ? attachments : undefined,
    };
    messages.value.push(userMsg);

    // Prepend file contents to the AI input
    let fullContent = content;
    if (attachments?.length) {
      const fileParts = attachments.map(a => `--- 文件: ${a.name} ---\n${a.content}`).join('\n\n');
      fullContent = fileParts + '\n\n' + content;
    }

    const aiMsgId = crypto.randomUUID();
    messages.value.push({
      id: aiMsgId,
      role: 'assistant',
      content: t('ai.analyzing'),
      timestamp: new Date().toISOString(),
      status: 'pending',
    });
    isProcessing.value = true;

    // Helper: get reactive proxy for the AI message (raw local var bypasses Vue)
    const getMsg = () => messages.value.find(m => m.id === aiMsgId);

    // Listen for real-time streaming events from the backend
    if (streamUnlisten) {
      streamUnlisten();
      streamUnlisten = null;
    }
    streamUnlisten = await listen<{ seq: number; event: { type: string; text: any } }>('ai-stream', (evt) => {
      if (evt.payload.seq !== seq) return;
      const msg = getMsg();
      if (!msg) return;

      const { type, text } = evt.payload.event;
      if (type === 'Reasoning') {
        if (!msg.reasoning) msg.reasoning = '';
        msg.reasoning += text;
        // Show "thinking" instead of static placeholder while reasoning streams in
        if (msg.content === t('ai.analyzing')) {
          msg.content = t('ai.thinkingStatus');
        }
      } else if (type === 'Fallback') {
        msg.fallbackInfo = { failed: text.failed, next: text.next };
      }
    });

    try {
      console.log('[AI] submitInput: calling invoke, seq=', seq);
      const result = await withTimeout(
        invoke<OrganizeResult>('ai_process_input', { content: fullContent, seq }),
        120_000,
      );
      console.log('[AI] submitInput: invoke returned', result);

      if (seq !== requestSeq) return;

      lastResult.value = result;
      const msg = getMsg();
      if (msg) {
        msg.content = result.complexity === 'simple' ? t('ai.autoProcessed') : t('ai.confirmPrompt');
        msg.reasoning = result.reasoning;
        msg.status = 'done';
        msg.suggestions = [{
          action: result.action as any,
          title: result.title,
          folder: result.folder,
          tags: result.tags,
          content: result.content,
          target_note_id: result.target_note_id || undefined,
          confidence: 0.9,
        }];
      }
    } catch (e: unknown) {
      if (seq !== requestSeq) return;

      const errMsg = extractError(e);
      console.error('[AI] submitInput failed:', e);
      const msg = getMsg();
      if (msg) {
        msg.content = t('ai.processFailed', { msg: errMsg });
        msg.status = 'error';
      }
    } finally {
      if (streamUnlisten) {
        streamUnlisten();
        streamUnlisten = null;
      }
      if (seq === requestSeq) {
        isProcessing.value = false;
      }
    }
  }

  function cancel() {
    requestSeq++;
    if (streamUnlisten) {
      streamUnlisten();
      streamUnlisten = null;
    }
    isProcessing.value = false;
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.status === 'pending') {
      lastMsg.content = t('ai.cancelled');
      lastMsg.status = 'error';
    }
  }

  async function applyResult(result: OrganizeResult) {
    // Clear suggestions immediately to prevent duplicate clicks
    const assistantMsg = messages.value.find(m => m.suggestions?.length);
    if (assistantMsg) {
      assistantMsg.suggestions = undefined;
    }
    lastResult.value = null;

    try {
      const note = await invoke<{ id: string }>('ai_apply_result', { result });

      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t(result.action === 'create' ? 'ai.noteCreated' : 'ai.noteUpdated', { title: result.title }),
        timestamp: new Date().toISOString(),
        status: 'done',
      });

      const notebookStore = useNotebookStore();
      await notebookStore.loadFolderTree();
      await notebookStore.loadAllNotes();
      if (note?.id) {
        await notebookStore.openNote(note.id);
      }
    } catch (e: unknown) {
      const errMsg = extractError(e);
      console.error('[AI] applyResult failed:', e);
      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t('ai.operationFailed', { msg: errMsg }),
        timestamp: new Date().toISOString(),
        status: 'error',
      });
    }
  }

  function dismiss() {
    lastResult.value = null;
    const last = messages.value[messages.value.length - 1];
    if (last) last.suggestions = [];
  }

  async function optimizeNote(noteId: string, instruction: string) {
    const seq = ++requestSeq;

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: instruction,
      timestamp: new Date().toISOString(),
      status: 'done',
    };
    messages.value.push(userMsg);

    const aiMsgId = crypto.randomUUID();
    messages.value.push({
      id: aiMsgId,
      role: 'assistant',
      content: t('ai.optimizing'),
      timestamp: new Date().toISOString(),
      status: 'pending',
    });
    isProcessing.value = true;

    const getMsg = () => messages.value.find(m => m.id === aiMsgId);

    if (streamUnlisten) { streamUnlisten(); streamUnlisten = null; }
    streamUnlisten = await listen<{ seq: number; event: { type: string; text: any } }>('ai-stream', (evt) => {
      if (evt.payload.seq !== seq) return;
      const msg = getMsg();
      if (!msg) return;
      const { type, text } = evt.payload.event;
      if (type === 'Reasoning') {
        if (!msg.reasoning) msg.reasoning = '';
        msg.reasoning += text;
        if (msg.content === t('ai.optimizing')) msg.content = t('ai.thinkingStatus');
      } else if (type === 'Fallback') {
        msg.fallbackInfo = { failed: text.failed, next: text.next };
      }
    });

    try {
      const result = await withTimeout(
        invoke<{ title: string; content: string; summary: string }>('ai_optimize_note', { noteId, instruction, seq }),
        120_000,
      );

      if (seq !== requestSeq) return;

      const msg = getMsg();
      if (msg) {
        msg.content = result.summary || t('ai.optimizeDone');
        msg.reasoning = undefined;
        msg.status = 'done';
        // Store optimize result as a special suggestion
        msg.suggestions = [{
          action: 'optimize' as any,
          title: result.title,
          content: result.content,
          target_note_id: noteId,
          confidence: 0.9,
        }];
      }
    } catch (e: unknown) {
      if (seq !== requestSeq) return;
      const errMsg = extractError(e);
      const msg = getMsg();
      if (msg) {
        msg.content = t('ai.optimizeFailed', { msg: errMsg });
        msg.status = 'error';
      }
    } finally {
      if (streamUnlisten) { streamUnlisten(); streamUnlisten = null; }
      if (seq === requestSeq) isProcessing.value = false;
    }
  }

  async function applyOptimize(noteId: string, title: string, content: string) {
    const assistantMsg = messages.value.find(m => m.suggestions?.length);
    if (assistantMsg) assistantMsg.suggestions = undefined;

    try {
      const notebookStore = useNotebookStore();
      await notebookStore.updateNoteContent(noteId, content);
      if (title) {
        await notebookStore.updateNoteContent(noteId, content, title);
      }
      await notebookStore.loadFolderTree();
      await notebookStore.loadAllNotes();

      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t('ai.optimizeApplied'),
        timestamp: new Date().toISOString(),
        status: 'done',
      });
    } catch (e: unknown) {
      const errMsg = extractError(e);
      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t('ai.applyFailed', { msg: errMsg }),
        timestamp: new Date().toISOString(),
        status: 'error',
      });
    }
  }

  function clearMessages() {
    messages.value = [];
    lastResult.value = null;
  }

  return { messages, isProcessing, lastResult, mode, submitInput, cancel, applyResult, dismiss, optimizeNote, applyOptimize, clearMessages };
});
