<template>
  <div class="folder-file-list">
    <div class="folder-header">
      <h3 class="folder-title">{{ folderName }}</h3>
      <span class="note-count">{{ notes.length }} 篇笔记</span>
    </div>
    <div v-if="notes.length === 0" class="empty-folder">
      <p>此目录下暂无笔记</p>
    </div>
    <div v-else class="note-cards">
      <div
        v-for="note in notes"
        :key="note.id"
        class="note-card"
        @click="handleClick(note)"
      >
        <div class="card-title">{{ note.title }}</div>
        <div class="card-summary">{{ note.summary || '暂无摘要' }}</div>
        <div v-if="note.tags.length" class="card-tags">
          <span v-for="tag in note.tags" :key="tag" class="card-tag">{{ tag }}</span>
        </div>
        <div class="card-footer">
          <span v-if="note.folder !== folderPath" class="card-folder">{{ note.folder }}</span>
          <span class="card-date">{{ formatDate(note.updated_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import type { Note } from '@/types/notebook';

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

const folderPath = computed(() => notebookStore.selectedFolderForList);
const folderName = computed(() => {
  const path = folderPath.value;
  return path ? path.split('/').pop() || path : '';
});
const notes = computed(() => notebookStore.folderNotes);

function formatDate(dateStr: string) {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  if (isNaN(d.getTime())) return dateStr;
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

async function handleClick(note: Note) {
  notebookStore.viewMode = 'editor';
  await notebookStore.openNote(note.id);
  if (notebookStore.currentNote) {
    editorStore.openTab(notebookStore.currentNote.note.id, notebookStore.currentNote.note.title);
  }
}
</script>

<style scoped>
.folder-file-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
  background: var(--bg-primary);
}

.folder-header {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 20px 28px 12px;
  border-bottom: 1px solid var(--border-color);
}

.folder-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.note-count {
  font-size: 13px;
  color: var(--text-secondary);
}

.empty-folder {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-secondary);
  font-size: 14px;
}

.note-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
  padding: 20px 28px;
}

.note-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: box-shadow 0.15s, border-color 0.15s;
  background: var(--bg-primary);
}

.note-card:hover {
  border-color: var(--accent-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-summary {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
}

.card-folder {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--bg-secondary);
  padding: 2px 8px;
  border-radius: 10px;
}

.card-date {
  font-size: 12px;
  color: var(--text-secondary);
}

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.card-tag {
  font-size: 11px;
  padding: 1px 6px;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}
</style>
