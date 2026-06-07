<template>
  <div class="search-panel" v-if="visible">
    <div class="search-header">
      <div class="search-input-wrap">
        <input
          v-model="query"
          placeholder="搜索笔记..."
          @input="handleSearch"
          ref="inputRef"
          @focus="showScopeHint = true"
          @blur="onInputBlur"
        />
        <button ref="scopeBtnRef" class="scope-btn" @mousedown.prevent="toggleScope">
          {{ scopeLabel }}
          <span class="scope-arrow">▾</span>
        </button>
      </div>
      <button class="close-btn" @click="emit('close')">×</button>
    </div>

    <div class="search-results">
      <div v-for="r in notebookStore.searchResults" :key="r.id" class="result-item" @click="openResult(r.id)">
        <div class="result-title">{{ r.title }}</div>
        <div v-if="r.snippet" class="result-snippet" v-html="r.snippet"></div>
        <div class="result-path">{{ r.path }}</div>
      </div>
      <div v-if="query && !debouncing && notebookStore.searchResults.length === 0" class="no-results">无结果</div>
    </div>

    <!-- Scope dropdown (rendered outside panel via Teleport) -->
    <Teleport to="body">
      <div
        v-if="scopeOpen"
        class="search-scope-dropdown"
        :style="{ top: dropdownPos.top + 'px', left: dropdownPos.left + 'px' }"
        @mousedown.prevent
        @click.stop
      >
        <button
          v-for="opt in scopeOptions"
          :key="opt.value"
          class="scope-option"
          :class="{ active: scope === opt.value }"
          @click="selectScope(opt.value)"
        >
          <span class="scope-icon">{{ opt.icon }}</span>
          {{ opt.label }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const query = ref('');
const scope = ref('all');
const scopeOpen = ref(false);
const showScopeHint = ref(false);
const debouncing = ref(false);
const inputRef = ref<HTMLInputElement>();
const scopeBtnRef = ref<HTMLButtonElement>();

const scopeOptions = [
  { value: 'all', label: '全部', icon: '🔍' },
  { value: 'title', label: '标题', icon: '📋' },
  { value: 'content', label: '内容', icon: '📝' },
  { value: 'tags', label: '标签', icon: '🏷' },
];

const scopeLabel = computed(() => scopeOptions.find(o => o.value === scope.value)?.label || '全部');

const dropdownPos = ref({ top: 0, left: 0 });

function updateDropdownPos() {
  const el = scopeBtnRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  dropdownPos.value = { top: rect.bottom + 4, left: rect.left };
}

let debounceTimer: ReturnType<typeof setTimeout>;

function handleSearch() {
  clearTimeout(debounceTimer);
  debouncing.value = true;
  debounceTimer = setTimeout(() => {
    debouncing.value = false;
    notebookStore.search(query.value, scope.value === 'all' ? undefined : scope.value);
  }, 300);
}

function toggleScope() {
  scopeOpen.value = !scopeOpen.value;
  if (scopeOpen.value) {
    nextTick(updateDropdownPos);
  }
}

function selectScope(value: string) {
  scope.value = value;
  scopeOpen.value = false;
  if (query.value) {
    notebookStore.search(query.value, value === 'all' ? undefined : value);
  }
}

function onInputBlur() {
  showScopeHint.value = false;
  // Close scope dropdown with delay to allow click events to fire
  setTimeout(() => { scopeOpen.value = false; }, 150);
}

async function openResult(id: string) {
  await notebookStore.openNote(id);
  const note = notebookStore.currentNote;
  if (note) {
    editorStore.openTab(note.note.id, note.note.title);
    editorStore.activeTabId = note.note.id;
  }
  emit('close');
}

watch(() => inputRef.value, (el) => {
  if (el) nextTick(() => el.focus());
});

onBeforeUnmount(() => clearTimeout(debounceTimer));
</script>

<style scoped>
.search-panel {
  position: fixed;
  top: 40px;
  left: 50%;
  transform: translateX(-50%);
  width: 520px;
  max-height: 420px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.18);
  z-index: 100;
  display: flex;
  flex-direction: column;
}
.search-header {
  display: flex;
  align-items: center;
  padding: 8px;
  gap: 4px;
  border-bottom: 1px solid var(--border-color);
}
.search-input-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-secondary);
}
.search-input-wrap input {
  flex: 1;
  border: none;
  outline: none;
  padding: 8px 10px;
  font-size: 14px;
  background: transparent;
  color: var(--text-primary);
  min-width: 0;
}
.search-input-wrap input::placeholder {
  color: var(--text-secondary);
}
.scope-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-primary);
  border: none;
  border-left: 1px solid var(--border-color);
  cursor: pointer;
  white-space: nowrap;
}
.scope-btn:hover {
  color: var(--accent-color);
}
.scope-arrow {
  font-size: 10px;
}
.close-btn {
  font-size: 18px;
  color: var(--text-secondary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0 8px;
}
.close-btn:hover {
  color: var(--text-primary);
}

.search-results {
  max-height: 340px;
  overflow-y: auto;
}
.result-item {
  padding: 10px 14px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-color);
}
.result-item:hover {
  background: var(--bg-secondary);
}
.result-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}
.result-snippet {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.5;
}
.result-snippet :deep(mark) {
  background: #fff3cd;
  color: inherit;
  padding: 0 2px;
  border-radius: 2px;
}
.result-path {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.7;
  margin-top: 2px;
}
.no-results {
  padding: 20px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}
</style>

<style>
.search-scope-dropdown {
  position: fixed;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  padding: 4px;
  z-index: 10000;
  min-width: 130px;
}
.search-scope-dropdown .scope-option {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  font-size: 13px;
  color: var(--text-primary);
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  text-align: left;
}
.search-scope-dropdown .scope-option:hover {
  background: var(--bg-secondary);
}
.search-scope-dropdown .scope-option.active {
  color: var(--accent-color);
  font-weight: 600;
}
.search-scope-dropdown .scope-icon {
  font-size: 14px;
}
</style>
