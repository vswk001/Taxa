// src/stores/ai.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ChatMessage, FileAttachment, StreamEventPayload } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import i18n from '@/i18n';
import { useNotebookStore } from './notebook';

/** Translate via the global i18n instance — works inside/outside setup. */
function t(key: string, named?: Record<string, unknown>): string {
  return i18n.global.t(key, named as any);
}

/** Current UI locale code (e.g. "en"), passed to the backend so the LLM
 *  outputs and reasons in the user's language. */
function currentLocale(): string {
  return (i18n.global.locale as any).value || 'zh-CN';
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

/** Race the promise against a timeout; when the timeout wins, tell the
 *  backend to actually abort the request (it honors ai_cancel). */
function withTimeout<T>(promise: Promise<T>, ms: number, seq: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      invoke('ai_cancel', { seq }).catch(() => { /* best effort */ });
      reject(new Error(t('ai.requestTimeout', { n: ms / 1000 })));
    }, ms);
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

  /** Subscribe to backend stream events for one request. The unlisten handle
   *  is request-local, so a finished/timed-out request can never unhook a
   *  newer request's listener. */
  async function listenStream(seq: number, onEvent: (evt: StreamEventPayload) => void): Promise<UnlistenFn> {
    return listen<{ seq: number; event: StreamEventPayload }>('ai-stream', (e) => {
      if (e.payload.seq !== seq) return;
      onEvent(e.payload.event);
    });
  }

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
      const fileParts = attachments.map(a => `--- ${t('ai.fileLabel')}: ${a.name} ---\n${a.content}`).join('\n\n');
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

    const unlisten = await listenStream(seq, ({ type, text }) => {
      const msg = getMsg();
      if (!msg || msg.status === 'error') return;
      if (type === 'Reasoning') {
        if (!msg.reasoning) msg.reasoning = '';
        msg.reasoning += text as string;
        // Show "thinking" instead of static placeholder while reasoning streams in
        if (msg.content === t('ai.analyzing')) {
          msg.content = t('ai.thinkingStatus');
        }
      } else if (type === 'Reset') {
        // The provider failed after streaming; drop its partial output.
        msg.reasoning = undefined;
      } else if (type === 'Fallback') {
        const info = text as { failed: string; next: string };
        msg.fallbackInfo = { failed: info.failed, next: info.next };
      }
    });

    try {
      const result = await withTimeout(
        invoke<OrganizeResult>('ai_process_input', { content: fullContent, seq, locale: currentLocale() }),
        120_000,
        seq,
      );

      if (seq !== requestSeq) return;

      lastResult.value = result;
      const msg = getMsg();
      if (msg) {
        msg.content = result.complexity === 'simple' ? t('ai.autoProcessed') : t('ai.confirmPrompt');
        msg.reasoning = result.reasoning;
        msg.status = 'done';
        msg.suggestions = [{
          action: result.action,
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
      const msg = getMsg();
      if (msg) {
        msg.content = t('ai.processFailed', { msg: errMsg });
        msg.status = 'error';
      }
    } finally {
      unlisten();
      if (seq === requestSeq) {
        isProcessing.value = false;
      }
    }
  }

  /** Mark the current request cancelled locally and abort it in the backend. */
  function cancel() {
    const seq = requestSeq;
    requestSeq++;
    invoke('ai_cancel', { seq }).catch(() => { /* best effort */ });
    isProcessing.value = false;
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.status === 'pending') {
      lastMsg.content = t('ai.cancelled');
      lastMsg.status = 'error';
    }
  }

  async function applyResult(result: OrganizeResult, msgId?: string) {
    // Clear the suggestions of the specific message (default: last with
    // suggestions) to prevent duplicate applies.
    const assistantMsg = msgId
      ? messages.value.find(m => m.id === msgId)
      : [...messages.value].reverse().find(m => m.suggestions?.length);
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
      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t('ai.operationFailed', { msg: errMsg }),
        timestamp: new Date().toISOString(),
        status: 'error',
      });
    }
  }

  function dismiss(msgId?: string) {
    lastResult.value = null;
    const msg = msgId
      ? messages.value.find(m => m.id === msgId)
      : messages.value[messages.value.length - 1];
    if (msg) msg.suggestions = [];
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

    const unlisten = await listenStream(seq, ({ type, text }) => {
      const msg = getMsg();
      if (!msg || msg.status === 'error') return;
      if (type === 'Reasoning') {
        if (!msg.reasoning) msg.reasoning = '';
        msg.reasoning += text as string;
        if (msg.content === t('ai.optimizing')) msg.content = t('ai.thinkingStatus');
      } else if (type === 'Reset') {
        msg.reasoning = undefined;
      } else if (type === 'Fallback') {
        const info = text as { failed: string; next: string };
        msg.fallbackInfo = { failed: info.failed, next: info.next };
      }
    });

    try {
      const result = await withTimeout(
        invoke<{ title: string; content: string; summary: string }>('ai_optimize_note', { noteId, instruction, seq, locale: currentLocale() }),
        120_000,
        seq,
      );

      if (seq !== requestSeq) return;

      const msg = getMsg();
      if (msg) {
        msg.content = result.summary || t('ai.optimizeDone');
        msg.reasoning = undefined;
        msg.status = 'done';
        // Store optimize result as a special suggestion
        msg.suggestions = [{
          action: 'optimize',
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
      unlisten();
      if (seq === requestSeq) isProcessing.value = false;
    }
  }

  async function applyOptimize(noteId: string, title: string, content: string, msgId?: string) {
    const assistantMsg = msgId
      ? messages.value.find(m => m.id === msgId)
      : [...messages.value].reverse().find(m => m.suggestions?.length);
    if (assistantMsg) assistantMsg.suggestions = undefined;

    try {
      const notebookStore = useNotebookStore();
      // One write with both fields; works whether or not the note is open.
      await notebookStore.updateNoteContent(noteId, content, title || undefined);
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
