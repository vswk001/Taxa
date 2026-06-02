<!-- src/components/ai/ChatArea.vue -->
<template>
  <div class="chat-area">
    <div v-if="messages.length === 0" class="empty-chat">
      <p>在这里输入内容，AI 将自动归类和完善</p>
    </div>
    <div v-for="msg in messages" :key="msg.id" :class="['message', msg.role]">
      <div class="message-content">{{ msg.content }}</div>
      <OperationCard
        v-if="msg.suggestions?.length"
        :suggestion="msg.suggestions[0]"
        @confirm="emit('apply', toResult(msg.suggestions[0]))"
        @dismiss="emit('dismiss')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ChatMessage, AiSuggestion } from '@/types/ai';
import type { OrganizeResult } from '@/types/ai-extended';
import OperationCard from './OperationCard.vue';

defineProps<{ messages: ChatMessage[] }>();
const emit = defineEmits<{ apply: [result: OrganizeResult]; dismiss: [] }>();

function toResult(s: AiSuggestion): OrganizeResult {
  return {
    action: s.action, title: s.title || '', folder: s.folder || '',
    tags: s.tags || [], content: s.content || '',
    target_note_id: s.target_note_id || null, complexity: 'complex',
  };
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
</style>
