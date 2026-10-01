<template>
  <div class="lib-card">
    <div v-for="(item, i) in items" :key="i" class="lib-item" :class="item.status">
      <span class="lib-icon">{{ icon(item.suggestion.type) }}</span>
      <div class="lib-main">
        <div class="lib-desc">{{ describe(item.suggestion) }}</div>
        <div class="lib-reason">{{ item.suggestion.reason }}</div>
      </div>
      <button v-if="item.status === 'pending'" class="lib-apply" @click="emit('apply', i)">{{ t('ai.libApply') }}</button>
      <button v-if="item.status === 'pending'" class="lib-skip" @click="emit('skip', i)">{{ t('ai.libSkip') }}</button>
      <span v-else-if="item.status === 'applying'" class="lib-status">…</span>
      <span v-else-if="item.status === 'done'" class="lib-status done">✓</span>
      <span v-else class="lib-status error">✗</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import type { LibrarySuggestion } from '@/types/ai-extended';

defineProps<{ items: { suggestion: LibrarySuggestion; status: 'pending' | 'applying' | 'done' | 'error' }[] }>();
const emit = defineEmits<{ apply: [index: number]; skip: [index: number] }>();
const { t } = useI18n();

function icon(kind: string): string {
  if (kind === 'move') return '📁';
  if (kind === 'merge') return '🔗';
  return '🏷';
}

function describe(s: LibrarySuggestion): string {
  const title = noteTitle(s.note_id);
  if (s.type === 'move') {
    return t('ai.libMove', { title, folder: s.target_folder ?? '' });
  }
  if (s.type === 'merge') {
    return t('ai.libMerge', { title, target: noteTitle(s.merge_with_id ?? '') });
  }
  return t('ai.libTag', { title, tags: s.tags.join(', ') });
}

// Resolve ids to titles via a map injected by the parent (avoids a store
// dependency in this presentational card).
const titleMap = defineModel<Record<string, string>>('titleMap', { default: () => ({}) });
function noteTitle(id: string): string {
  return titleMap.value[id] ?? id.slice(0, 8);
}
</script>

<style scoped>
.lib-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 8px;
}

.lib-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-primary);
}

.lib-item.done {
  opacity: 0.55;
}

.lib-icon {
  font-size: 15px;
  flex-shrink: 0;
}

.lib-main {
  flex: 1;
  min-width: 0;
}

.lib-desc {
  font-size: 12px;
  color: var(--text-primary);
}

.lib-reason {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.lib-apply,
.lib-skip {
  padding: 3px 10px;
  font-size: 11px;
  border-radius: 5px;
  border: 1px solid var(--border-color);
  background: none;
  cursor: pointer;
  color: var(--text-primary);
  flex-shrink: 0;
}

.lib-apply {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}

.lib-apply:hover,
.lib-skip:hover {
  opacity: 0.85;
}

.lib-status {
  font-size: 12px;
  flex-shrink: 0;
}

.lib-status.done {
  color: var(--accent-color);
}

.lib-status.error {
  color: var(--danger-color);
}
</style>
