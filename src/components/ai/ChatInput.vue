<!-- src/components/ai/ChatInput.vue -->
<template>
  <div class="chat-input">
    <textarea
      v-model="text"
      placeholder="输入内容或拖入文件..."
      :disabled="disabled"
      @keydown.enter.exact.prevent="submit"
      @drop.prevent="handleDrop"
    />
    <button class="send-btn" :disabled="disabled || !text.trim()" @click="submit">发送</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ submit: [content: string] }>();
const text = ref('');

function submit() {
  if (text.value.trim()) {
    emit('submit', text.value.trim());
    text.value = '';
  }
}

function handleDrop(e: DragEvent) {
  const files = e.dataTransfer?.files;
  if (files?.length) {
    text.value += `\n[文件: ${Array.from(files).map(f => f.name).join(', ')}]`;
  }
}
</script>

<style scoped>
.chat-input {
  display: flex; gap: 8px; padding: 12px;
  border-top: 1px solid var(--border-color);
}
.chat-input textarea {
  flex: 1; resize: none; border: 1px solid var(--border-color);
  border-radius: var(--radius); padding: 8px; font-size: 13px;
  background: var(--bg-primary); color: var(--text-primary);
  min-height: 40px; max-height: 120px;
}
.chat-input textarea:focus { outline: none; border-color: var(--accent-color); }
.send-btn {
  padding: 8px 16px; background: var(--accent-color); color: white;
  border-radius: var(--radius); font-size: 13px;
}
.send-btn:hover:not(:disabled) { background: var(--accent-hover); }
.send-btn:disabled { opacity: 0.5; }
</style>
