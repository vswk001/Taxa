<template>
  <Teleport to="body">
    <div v-if="visible" class="trash-overlay" @click.self="emit('close')">
      <div class="trash-dialog">
        <div class="trash-header">
          <span>{{ t('trash.title') }}</span>
          <div class="header-actions">
            <button
              v-if="items.length"
              class="empty-btn danger"
              @click="emptyAll"
            >{{ t('trash.emptyAll') }}</button>
            <button class="close-btn" @click="emit('close')">×</button>
          </div>
        </div>
        <div class="trash-list">
          <div v-if="!items.length" class="empty-state">{{ t('trash.empty') }}</div>
          <div v-for="item in items" :key="item.id" class="trash-item">
            <div class="item-info">
              <span class="item-title">{{ item.title }}</span>
              <span class="item-folder">{{ item.folder || t('tree.Uncategorized') }}</span>
              <span class="item-date">{{ formatDate(item.deleted_at) }}</span>
            </div>
            <div class="item-actions">
              <button @click="restore(item.id)">{{ t('trash.restore') }}</button>
              <button class="danger" @click="purge(item.id)">{{ t('trash.deleteForever') }}</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { message as tauriMessage } from '@tauri-apps/plugin-dialog';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import type { TrashItem } from '@/types/notebook';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const items = ref<TrashItem[]>([]);

async function refresh() {
  try {
    items.value = await invoke<TrashItem[]>('list_trash');
  } catch (e) {
    console.error('failed to list trash:', e);
  }
}

watch(
  () => props.visible,
  (v) => {
    if (v) refresh();
  },
  { immediate: true },
);

async function restore(id: string) {
  try {
    const note = await invoke<{ id: string; title: string }>('restore_note', { id });
    await notebookStore.loadFolderTree();
    await notebookStore.loadAllNotes();
    editorStore.openTab(note.id, note.title);
    await refresh();
  } catch (e) {
    await tauriMessage(e instanceof Error ? e.message : String(e), {
      title: t('trash.restoreFailed'),
      kind: 'error',
    });
  }
}

async function purge(id: string) {
  try {
    await invoke('purge_note', { id });
    await refresh();
  } catch (e) {
    await tauriMessage(e instanceof Error ? e.message : String(e), {
      title: t('common.unknownError'),
      kind: 'error',
    });
  }
}

async function emptyAll() {
  if (!items.value.length) return;
  try {
    await invoke('empty_trash');
    items.value = [];
    await notebookStore.loadFolderTree();
  } catch (e) {
    await tauriMessage(e instanceof Error ? e.message : String(e), {
      title: t('common.unknownError'),
      kind: 'error',
    });
  }
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '';
  return dateStr.slice(0, 16).replace('T', ' ');
}
</script>

<style scoped>
.trash-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 150;
  display: flex;
  align-items: center;
  justify-content: center;
}

.trash-dialog {
  width: 520px;
  max-height: 480px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.trash-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.empty-btn,
.close-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
}

.empty-btn.danger {
  color: var(--danger-color);
}

.close-btn {
  font-size: 18px;
}

.trash-list {
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

.trash-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-color);
}

.trash-item:last-child {
  border-bottom: none;
}

.item-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.item-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-folder {
  font-size: 11px;
  color: var(--text-secondary);
}

.item-date {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.7;
  margin-left: auto;
}

.item-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.item-actions button {
  padding: 3px 10px;
  font-size: 12px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-primary);
}

.item-actions button:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.item-actions button.danger:hover {
  border-color: var(--danger-color);
  color: var(--danger-color);
}
</style>
