<template>
  <div class="chat-area">
    <div v-if="messages.length === 0" class="empty-chat">
      <p>{{ t('ai.emptyChat') }}</p>
    </div>
    <div v-for="msg in messages" :key="msg.id" :class="['message', msg.role]">
      <div v-if="msg.reasoning" class="thinking-card" :class="{ collapsed: !expandedMap[msg.id] }">
        <div class="thinking-header" @click="expandedMap[msg.id] = !expandedMap[msg.id]">
          <svg class="chevron" :class="{ rotated: expandedMap[msg.id] }" viewBox="0 0 24 24" width="14" height="14"><polyline points="6 9 12 15 18 9" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>
          <span class="thinking-label">{{ t('ai.thinkingProcess') }}</span>
          <span class="thinking-duration" v-if="!expandedMap[msg.id]">{{ preview(msg.reasoning) }}</span>
        </div>
        <div v-if="expandedMap[msg.id]" class="thinking-body">
          <div class="thinking-content">{{ msg.reasoning }}</div>
        </div>
      </div>
      <div v-if="msg.fallbackInfo" class="fallback-note">
        {{ t('ai.fallbackNotice', { failed: msg.fallbackInfo.failed, next: msg.fallbackInfo.next }) }}
      </div>
      <div class="message-content">{{ msg.content }}</div>
      <div v-if="msg.attachments?.length" class="msg-attachments">
        <span v-for="a in msg.attachments" :key="a.name" class="msg-file-chip">📎 {{ a.name }}</span>
      </div>
      <OperationCard
        v-if="msg.suggestions?.length"
        :suggestion="msg.suggestions[0]"
        @confirm="handleConfirm(msg.suggestions[0], $event)"
        @dismiss="emit('dismiss')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ChatMessage, AiSuggestion } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import OperationCard from './OperationCard.vue';

const { t } = useI18n();

defineProps<{ messages: ChatMessage[] }>();
const emit = defineEmits<{ apply: [result: OrganizeResult]; applyOptimize: [noteId: string, title: string, content: string]; dismiss: [] }>();
const expandedMap = reactive<Record<string, boolean>>({});

function preview(text: string) {
  const first = text.split('\n')[0] || '';
  return first.length > 40 ? first.slice(0, 40) + '...' : first;
}

function toResult(s: AiSuggestion): OrganizeResult {
  return {
    action: s.action, title: s.title || '', folder: s.folder || '',
    tags: s.tags || [], content: s.content || '',
    target_note_id: s.target_note_id || null, complexity: 'complex',
  };
}

function handleConfirm(original: AiSuggestion, modified: AiSuggestion) {
  if (modified.action === 'optimize') {
    const noteId = original.target_note_id || modified.target_note_id || '';
    emit('applyOptimize', noteId, modified.title || '', modified.content || '');
  } else {
    emit('apply', toResult(modified));
  }
}
</script>

<style scoped>
.chat-area { flex: 1; overflow-y: auto; padding: 12px; }
.empty-chat { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-secondary); font-size: 13px; text-align: center; }
.message { margin-bottom: 12px; padding: 8px 12px; border-radius: 8px; font-size: 13px; line-height: 1.5; }
.message.user { background: var(--accent-color); color: white; }
.message.assistant { background: var(--bg-secondary); }
.message.system { background: var(--bg-secondary); font-style: italic; color: var(--text-secondary); font-size: 12px; }
.message-content { white-space: pre-wrap; }
.msg-attachments { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; }
.msg-file-chip {
  font-size: 11px; padding: 2px 8px; border-radius: 10px;
  background: rgba(255,255,255,0.15);
}
.message:not(.user) .msg-file-chip { background: var(--bg-secondary); }
.fallback-note {
  font-size: 11px; padding: 3px 8px; margin-bottom: 6px;
  border-radius: 4px; color: #b8860b;
  background: rgba(184,134,11,0.1); border: 1px solid rgba(184,134,11,0.25);
}

/* Thinking card */
.thinking-card {
  margin-bottom: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-primary);
}
.thinking-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  cursor: pointer;
  user-select: none;
  font-size: 12px;
  color: var(--text-secondary);
  transition: background 0.15s;
}
.thinking-header:hover {
  background: var(--bg-secondary);
}
.chevron {
  flex-shrink: 0;
  transition: transform 0.2s;
}
.chevron.rotated {
  transform: rotate(180deg);
}
.thinking-label {
  font-weight: 600;
  color: var(--accent-color);
  flex-shrink: 0;
}
.thinking-duration {
  color: var(--text-secondary);
  opacity: 0.7;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.thinking-body {
  padding: 0 10px 8px;
}
.thinking-content {
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-secondary);
  white-space: pre-wrap;
  max-height: 300px;
  overflow-y: auto;
  padding: 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
}
</style>
