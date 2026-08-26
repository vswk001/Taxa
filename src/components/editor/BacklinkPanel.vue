<template>
  <div v-if="open" class="backlink-panel">
    <div class="panel-header">
      <span>{{ t('editor.backlinks') }}</span>
      <div class="header-actions">
        <span v-if="count" class="count">{{ count }}</span>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>
    </div>

    <div v-if="!count" class="empty">{{ t('editor.noLinks') }}</div>
    <template v-else>
      <div v-if="links.backlinks.length" class="section">
        <div class="section-title">{{ t('editor.linkedIn') }}</div>
        <button v-for="item in links.backlinks" :key="'b' + item.id" class="link-item" @click="openNote(item.id)">
          <span class="item-title">{{ item.title }}</span>
          <span class="item-folder">{{ item.folder || t('tree.Uncategorized') }}</span>
        </button>
      </div>
      <div v-if="links.outgoing.length" class="section">
        <div class="section-title">{{ t('editor.linkedOut') }}</div>
        <button v-for="item in links.outgoing" :key="'o' + item.id" class="link-item" @click="openNote(item.id)">
          <span class="item-title">{{ item.title }}</span>
          <span class="item-folder">{{ item.folder || t('tree.Uncategorized') }}</span>
        </button>
      </div>
      <div v-if="links.unresolved.length" class="section">
        <div class="section-title">{{ t('editor.unresolvedLinks') }}</div>
        <button v-for="title in links.unresolved" :key="'u' + title" class="link-item unresolved" @click="createLinked(title)">
          <span class="item-title">{{ title }}</span>
          <span class="create-hint">{{ t('editor.createNote') }}</span>
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import type { NoteLinks } from '@/types/notebook';

const props = defineProps<{ open: boolean; noteId: string | null; noteFolder: string }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const links = ref<NoteLinks>({ backlinks: [], outgoing: [], unresolved: [] });

const count = computed(
  () => links.value.backlinks.length + links.value.outgoing.length + links.value.unresolved.length,
);

async function refresh() {
  if (!props.noteId) {
    links.value = { backlinks: [], outgoing: [], unresolved: [] };
    return;
  }
  try {
    links.value = await invoke<NoteLinks>('get_note_links', { id: props.noteId });
  } catch (e) {
    console.error('failed to load note links:', e);
  }
}

// Reload when the panel opens or the note (content) changes.
watch(
  () => [props.open, props.noteId, props.open ? notebookStore.currentNote?.content : ''] as const,
  () => {
    if (props.open) refresh();
  },
);

async function openNote(id: string) {
  await notebookStore.openNote(id);
  const note = notebookStore.currentNote;
  if (note) editorStore.openTab(note.note.id, note.note.title);
}

async function createLinked(title: string) {
  try {
    const note = await notebookStore.createNote(props.noteFolder, title, `# ${title}\n`);
    editorStore.openTab(note.id, note.title);
  } catch (e) {
    console.error('failed to create linked note:', e);
  }
}
</script>

<style scoped>
.backlink-panel {
  position: absolute;
  top: 48px;
  right: 12px;
  width: 280px;
  max-height: 60%;
  overflow-y: auto;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  z-index: 50;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.count {
  font-size: 11px;
  background: var(--accent-color);
  color: white;
  border-radius: 8px;
  padding: 1px 7px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  color: var(--text-secondary);
}

.empty {
  padding: 20px;
  text-align: center;
  font-size: 12px;
  color: var(--text-secondary);
}

.section {
  padding: 8px 6px;
}

.section + .section {
  border-top: 1px solid var(--border-color);
}

.section-title {
  font-size: 11px;
  color: var(--text-secondary);
  padding: 2px 8px 6px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.link-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
}

.link-item:hover {
  background: var(--bg-secondary);
}

.item-title {
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-folder {
  font-size: 10px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.link-item.unresolved .item-title {
  color: var(--text-secondary);
  font-style: italic;
}

.create-hint {
  font-size: 10px;
  color: var(--accent-color);
  flex-shrink: 0;
}
</style>
