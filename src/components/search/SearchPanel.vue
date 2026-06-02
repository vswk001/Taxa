<!-- src/components/search/SearchPanel.vue -->
<template>
  <div class="search-panel" v-if="visible">
    <div class="search-header">
      <input v-model="query" placeholder="搜索笔记..." @input="handleSearch" ref="inputRef" />
      <button @click="emit('close')">×</button>
    </div>
    <div class="search-results">
      <div v-for="r in notebookStore.searchResults" :key="r.id" class="result-item" @click="openResult(r.id)">
        <div class="result-title">{{ r.title }}</div>
        <div class="result-snippet" v-html="r.snippet"></div>
      </div>
      <div v-if="query && notebookStore.searchResults.length === 0" class="no-results">无结果</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useNotebookStore } from '@/stores/notebook';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const notebookStore = useNotebookStore();
const query = ref('');
const inputRef = ref<HTMLInputElement>();

let debounceTimer: ReturnType<typeof setTimeout>;
function handleSearch() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => notebookStore.search(query.value), 300);
}

function openResult(id: string) {
  notebookStore.openNote(id);
  emit('close');
}

watch(() => inputRef.value, (el) => { if (el) el.focus(); });
</script>

<style scoped>
.search-panel {
  position: fixed; top: 40px; left: 50%; transform: translateX(-50%);
  width: 500px; max-height: 400px; background: var(--bg-primary);
  border: 1px solid var(--border-color); border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.15); z-index: 100; overflow: hidden;
}
.search-header { display: flex; align-items: center; padding: 8px; border-bottom: 1px solid var(--border-color); }
.search-header input { flex: 1; border: none; outline: none; padding: 8px; font-size: 14px; background: transparent; }
.search-header button { font-size: 18px; color: var(--text-secondary); }
.search-results { max-height: 340px; overflow-y: auto; }
.result-item { padding: 10px 14px; cursor: pointer; border-bottom: 1px solid var(--border-color); }
.result-item:hover { background: var(--bg-secondary); }
.result-title { font-weight: 600; font-size: 14px; }
.result-snippet { font-size: 12px; color: var(--text-secondary); margin-top: 4px; }
.result-snippet :deep(mark) { background: #fff3cd; color: inherit; padding: 0 2px; }
.no-results { padding: 20px; text-align: center; color: var(--text-secondary); }
</style>
