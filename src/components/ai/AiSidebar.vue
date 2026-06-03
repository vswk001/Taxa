<template>
  <div class="ai-sidebar">
    <div class="sidebar-header">
      <span>AI 助手</span>
      <div class="header-actions">
        <button v-if="aiStore.messages.length > 0" @click="aiStore.clearMessages()" title="清空对话">🗑</button>
      </div>
    </div>
    <ChatArea :messages="aiStore.messages" @apply="aiStore.applyResult($event)" @dismiss="aiStore.dismiss()" />
    <div v-if="aiStore.isProcessing" class="processing-bar">
      <span class="processing-text">AI 处理中...</span>
      <button class="cancel-btn" @click="aiStore.cancel()">取消</button>
    </div>
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
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-actions button {
  background: none; border: none; cursor: pointer; font-size: 14px;
  color: var(--text-secondary); padding: 2px 6px; border-radius: 4px;
}
.header-actions button:hover { background: var(--border-color); }

.processing-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 8px 12px; background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
}
.processing-text { font-size: 12px; color: var(--text-secondary); }
.cancel-btn {
  padding: 4px 12px; font-size: 12px; background: var(--danger-color);
  color: white; border: none; border-radius: 4px; cursor: pointer;
}
.cancel-btn:hover { opacity: 0.9; }
</style>
