// src/stores/ai.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { ChatMessage } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import { invoke } from '@tauri-apps/api/core';

const AI_TIMEOUT_MS = 60000;

export const useAiStore = defineStore('ai', () => {
  const messages = ref<ChatMessage[]>([]);
  const isProcessing = ref(false);
  const lastResult = ref<OrganizeResult | null>(null);

  async function submitInput(content: string) {
    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
      status: 'done',
    };
    messages.value.push(userMsg);

    const aiMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'assistant',
      content: '正在分析...',
      timestamp: new Date().toISOString(),
      status: 'pending',
    };
    messages.value.push(aiMsg);
    isProcessing.value = true;

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), AI_TIMEOUT_MS);

    try {
      const result = await invoke<OrganizeResult>('ai_process_input', { content });
      clearTimeout(timeoutId);
      lastResult.value = result;
      aiMsg.content = result.complexity === 'simple' ? '已自动处理' : '请确认以下操作';
      aiMsg.status = 'done';
      aiMsg.suggestions = [{
        action: result.action as any,
        title: result.title,
        folder: result.folder,
        tags: result.tags,
        content: result.content,
        target_note_id: result.target_note_id || undefined,
        confidence: 0.9,
      }];
    } catch (e: any) {
      clearTimeout(timeoutId);
      if (e === 'Aborted' || e?.name === 'AbortError') {
        aiMsg.content = '请求超时，请稍后重试';
      } else {
        aiMsg.content = `处理失败: ${e}`;
      }
      aiMsg.status = 'error';
    } finally {
      isProcessing.value = false;
    }
  }

  function cancel() {
    isProcessing.value = false;
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg && lastMsg.status === 'pending') {
      lastMsg.content = '已取消';
      lastMsg.status = 'error';
    }
  }

  async function applyResult(result: OrganizeResult) {
    try {
      await invoke('ai_apply_result', { result });
      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: `已${result.action === 'create' ? '创建' : '更新'}笔记: ${result.title}`,
        timestamp: new Date().toISOString(),
        status: 'done',
      });
      lastResult.value = null;
    } catch (e: any) {
      console.error('Apply failed:', e);
      messages.value.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: `操作失败: ${e}`,
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
