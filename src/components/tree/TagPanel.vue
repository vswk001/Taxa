<template>
  <Teleport to="body">
    <div v-if="visible" class="tag-overlay" @click.self="emit('close')">
      <div class="tag-dialog">
        <div class="tag-header">
          <span>{{ t('tags.title') }}</span>
          <button class="close-btn" @click="emit('close')">×</button>
        </div>
        <div class="tag-list">
          <div v-if="!tags.length" class="empty-state">{{ t('tags.empty') }}</div>
          <div v-for="[tag, count] in tags" :key="tag" class="tag-row">
            <template v-if="renaming === tag">
              <input
                ref="renameInputRef"
                v-model="renameValue"
                class="rename-input"
                @keydown.enter.prevent="commitRename(tag)"
                @keydown.escape="cancelRename"
                @blur="commitRename(tag)"
              />
            </template>
            <template v-else>
              <button class="tag-main" @click="searchTag(tag)">
                <span class="tag-name">{{ tag }}</span>
                <span class="tag-count">{{ count }}</span>
              </button>
              <button class="rename-btn" :title="t('tags.rename')" @click="startRename(tag)">✎</button>
            </template>
          </div>
        </div>
        <div v-if="renaming" class="tag-footer">{{ t('tags.renameHint') }}</div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNotebookStore } from '@/stores/notebook';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const notebookStore = useNotebookStore();

/** Tags sorted by usage; clicking one jumps to a scoped search. */
const tags = computed(() => {
  const counts = new Map<string, number>();
  for (const note of notebookStore.notes) {
    for (const tag of note.tags ?? []) {
      counts.set(tag, (counts.get(tag) ?? 0) + 1);
    }
  }
  return [...counts.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
});

function searchTag(tag: string) {
  // AppLayout listens and opens the search panel prefilled for this tag.
  window.dispatchEvent(new CustomEvent('taxa:search-tag', { detail: tag }));
  emit('close');
}

// ---- global rename ------------------------------------------------------
// Renames the tag across every note that has it. updateNoteTags omits
// content for non-open notes, so files are never touched by this loop.
const renaming = ref<string | null>(null);
const renameValue = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

function startRename(tag: string) {
  renaming.value = tag;
  renameValue.value = tag;
  void nextTick(() => renameInputRef.value?.focus());
}

function cancelRename() {
  renaming.value = null;
}

async function commitRename(oldTag: string) {
  if (renaming.value !== oldTag) return;
  const newTag = renameValue.value.trim();
  renaming.value = null;
  if (!newTag || newTag === oldTag) return;

  const affected = notebookStore.notes.filter(
    (n) => n.tags?.includes(oldTag) && !(n.tags.includes(newTag) && oldTag !== newTag),
  );
  for (const note of affected) {
    const next = note.tags.includes(newTag)
      ? note.tags.filter((t2) => t2 !== oldTag) // merge into existing tag
      : note.tags.map((t2) => (t2 === oldTag ? newTag : t2));
    try {
      await notebookStore.updateNoteTags(note.id, next);
    } catch (e) {
      console.error(`failed to rename tag on note ${note.id}:`, e);
    }
  }
}

watch(
  () => props.visible,
  (v) => {
    if (!v) cancelRename();
  },
);
</script>

<style scoped>
.tag-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 150;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tag-dialog {
  width: 420px;
  max-height: 460px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tag-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-secondary);
}

.tag-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.empty-state {
  padding: 32px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 4px;
}

.tag-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 7px 10px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
}

.tag-main:hover {
  background: var(--bg-secondary);
}

.tag-name {
  font-size: 13px;
  color: var(--text-primary);
}

.tag-count {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  border-radius: 9px;
  padding: 1px 8px;
}

.tag-main:hover .tag-count {
  background: var(--bg-primary);
}

.rename-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  padding: 4px 8px;
  border-radius: 5px;
  flex-shrink: 0;
}

.rename-btn:hover {
  color: var(--accent-color);
  background: var(--bg-secondary);
}

.rename-input {
  flex: 1;
  padding: 6px 10px;
  font-size: 13px;
  border: 1px solid var(--accent-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
}

.tag-footer {
  padding: 8px 16px;
  font-size: 11px;
  color: var(--text-secondary);
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}
</style>
