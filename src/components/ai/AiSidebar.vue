<!-- src/components/ai/AiSidebar.vue -->
<template>
  <div class="ai-sidebar">
    <div class="sidebar-header">
      <span>AI 助手</span>
    </div>
    <ChatArea :messages="aiStore.messages" @apply="aiStore.applyResult($event)" @dismiss="aiStore.dismiss()" />
    <ChatInput :disabled="aiStore.isProcessing" @submit="aiStore.submitInput($event)" />
  </div>
</template>

<script setup lang="ts">
import { useAiStore } from '@/stores/ai';
import ChatArea from './ChatArea.vue';
import ChatInput from './ChatInput.vue';

const aiStore = useAiStore();
</script>

<style scoped>
.ai-sidebar {
  width: var(--ai-sidebar-width);
  min-width: var(--ai-sidebar-width);
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-left: 1px solid var(--border-color);
}

.sidebar-header {
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}
</style>
