<!-- src/components/layout/StatusBar.vue -->
<template>
  <div class="status-bar">
    <span class="status-item">{{ notebookStore.notes.length }} 笔记</span>
    <span class="status-item">{{ defaultProvider || '未配置 LLM' }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useSettingsStore } from '@/stores/settings';

const notebookStore = useNotebookStore();
const settingsStore = useSettingsStore();

const defaultProvider = computed(() => {
  const p = settingsStore.providers.find(p => p.is_default && p.enabled);
  return p ? `${p.name} (${p.model_name})` : null;
});
</script>

<style scoped>
.status-bar {
  height: var(--status-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--bg-sidebar);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
