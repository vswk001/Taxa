// src/stores/ai.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ChatMessage } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useNotebookStore } from './notebook';

function extractError(e: unknown): string {
  if (typeof e === 'string') return e;
  if (e instanceof Error) return e.message;
  if (e && typeof e === 'object') {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === 'string') return obj.message;
    if (typeof obj.cause === 'string') return obj.cause;
    try { return JSON.stringify(e); } catch { /* fall through */ }
  }
  return '未知错误';
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`请求超时 (${ms / 1000}秒)`)), ms);
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
  let requestSeq = 0;
  let streamUnlisten: UnlistenFn | null = null;

  async function submitInput(content: string) {
    const seq = ++requestSeq;

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
      status: 'done',
    };
    messages.value.push(userMsg);

    const aiMsgId = crypto.randomUUID();
    messages.value.push({
      id: aiMsgId,
      role: 'assistant',
      content: '正在分析...',
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
    streamUnlisten = await listen<{ seq: number; event: { type: string; text: string } }>('ai-stream', (evt) => {
      if (evt.payload.seq !== seq) return;
      const msg = getMsg();
      if (!msg) return;

      const { type, text } = evt.payload.event;
      if (type === 'Reasoning') {
        if (!msg.reasoning) msg.reasoning = '';
        msg.reasoning += text;
        // Show "thinking" instead of static placeholder while reasoning streams in
        if (msg.content === '正在分析...') {
          msg.content = '正在思考...';
        }
      }
    });

    try {
      console.log('[AI] submitInput: calling invoke, seq=', seq);
      const result = await withTimeout(
        invoke<OrganizeResult>('ai_process_input', { content, seq }),
        120_000,
      );
      console.log('[AI] submitInput: invoke returned', result);

      if (seq !== requestSeq) return;

      lastResult.value = result;
      const msg = getMsg();
      if (msg) {
        msg.content = result.complexity === 'simple' ? '已自动处理' : '请确认以下操作';
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
        msg.content = `处理失败: ${errMsg}`;
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
      lastMsg.content = '已取消';
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
        content: `已${result.action === 'create' ? '创建' : '更新'}笔记: ${result.title}`,
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
        content: `操作失败: ${errMsg}`,
        timestamp: new Date().toISOString(),
        status: 'error',
      });
    }
  }

  function dismiss() {
    lastResult.value = null;
  }

  function clearMessages() {
    messages.value = [];
    lastResult.value = null;
  }

  return { messages, isProcessing, lastResult, submitInput, cancel, applyResult, dismiss, clearMessages };
});
