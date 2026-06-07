<template>
  <div class="note-editor" @keydown="handleKeyDown">
    <div v-if="notebookStore.currentNote" class="note-header">
      <div class="header-row">
        <input
          v-model="localTitle"
          class="title-input"
          placeholder="笔记标题..."
          @blur="saveTitle"
          @keyup.enter="saveTitle"
        />
        <span class="note-meta">{{ wordCount }} 字</span>
      </div>
      <div class="tag-row">
        <span
          v-for="tag in localTags"
          :key="tag"
          class="tag-badge"
        >
          {{ tag }}
          <button class="tag-remove" @click="removeTag(tag)">×</button>
        </span>
        <div v-if="tagInputVisible" class="tag-input-wrap">
          <input
            ref="tagInputRef"
            v-model="newTag"
            class="tag-input"
            placeholder="标签名"
            @keydown.enter.prevent="addTag"
            @keydown.escape="tagInputVisible = false"
            @blur="addTag"
          />
        </div>
        <button v-else class="tag-add" @click="showTagInput">+</button>
      </div>
    </div>
    <div v-else class="note-header">
      <span class="no-note-message">选择或创建一个笔记开始编辑</span>
    </div>

    <EditorSearch
      :visible="searchVisible"
      :container="editorContainer"
      @close="searchVisible = false"
    />

    <div v-if="notebookStore.currentNote" ref="editorContainer" class="editor-body">
      <MilkdownEditor v-model="localContent" />
    </div>
    <div v-else class="editor-placeholder">
      <div class="placeholder-content">
        <p>暂无选中的笔记</p>
        <p class="hint">从左侧选择笔记，或点击 "+" 创建新笔记</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onBeforeUnmount, nextTick } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import MilkdownEditor from './MilkdownEditor.vue';
import EditorSearch from './EditorSearch.vue';

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const localTitle = ref('');
const localContent = ref('');
const localTags = ref<string[]>([]);
const tagInputVisible = ref(false);
const newTag = ref('');
const tagInputRef = ref<HTMLInputElement | null>(null);
const searchVisible = ref(false);
const editorContainer = ref<HTMLElement | null>(null);
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let saveSeq = 0;
let isLoadingNote = false;

const wordCount = computed(() => {
  if (!localContent.value) return 0;
  const text = localContent.value.trim();
  if (!text) return 0;
  const chineseChars = text.match(/[\u4e00-\u9fa5]/g)?.length || 0;
  const englishWords = text.replace(/[\u4e00-\u9fa5]/g, ' ').match(/[a-zA-Z]+/g)?.length || 0;
  return chineseChars + englishWords;
});

watch(() => notebookStore.currentNote, (newNote) => {
  if (newNote) {
    isLoadingNote = true;
    localTitle.value = newNote.note.title;
    localContent.value = newNote.content;
    localTags.value = [...newNote.note.tags];
    editorStore.openTab(newNote.note.id, newNote.note.title);
    // Let the content prop propagate to Milkdown, then clear the loading flag
    setTimeout(() => { isLoadingNote = false; }, 0);
  } else {
    isLoadingNote = false;
    localTitle.value = '';
    localContent.value = '';
    localTags.value = [];
  }
}, { immediate: true });

watch(() => editorStore.activeTabId, async (newTabId) => {
  if (newTabId && notebookStore.currentNote?.note.id !== newTabId) {
    await flushSave();
    await notebookStore.openNote(newTabId);
  }
});

watch(localContent, () => {
  if (isLoadingNote) return;
  handleContentChange();
});

function handleContentChange() {
  if (saveTimer) clearTimeout(saveTimer);
  const seq = ++saveSeq;
  const p = new Promise<void>((resolve) => {
    saveTimer = setTimeout(async () => {
      if (seq === saveSeq) {
        await saveContent();
      }
      resolve();
    }, 1000);
  });
  editorStore.setSavePromise(p);
}

async function flushSave() {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  await saveContent();
  await editorStore.waitForSave();
}

async function saveContent() {
  if (notebookStore.currentNote && localContent.value !== undefined) {
    try {
      await notebookStore.updateNoteContent(notebookStore.currentNote.note.id, localContent.value);
    } catch (error) {
      console.error('Failed to save content:', error);
    }
  }
}

async function saveTitle() {
  if (notebookStore.currentNote && localTitle.value.trim() && localTitle.value !== notebookStore.currentNote.note.title) {
    try {
      const noteId = notebookStore.currentNote.note.id;
      const newTitle = localTitle.value.trim();
      await notebookStore.updateNoteContent(noteId, localContent.value, newTitle);
      editorStore.updateTabTitle(noteId, newTitle);
    } catch (error) {
      console.error('Failed to save title:', error);
    }
  }
}

function showTagInput() {
  tagInputVisible.value = true;
  newTag.value = '';
  nextTick(() => tagInputRef.value?.focus());
}

function addTag() {
  const t = newTag.value.trim();
  tagInputVisible.value = false;
  if (!t || localTags.value.includes(t)) { newTag.value = ''; return; }
  localTags.value.push(t);
  newTag.value = '';
  saveTags();
}

function removeTag(tag: string) {
  localTags.value = localTags.value.filter(t => t !== tag);
  saveTags();
}

async function saveTags() {
  if (notebookStore.currentNote) {
    try {
      await notebookStore.updateNoteTags(notebookStore.currentNote.note.id, [...localTags.value]);
    } catch (error) {
      console.error('Failed to save tags:', error);
    }
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault();
    flushSave();
  } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
    event.preventDefault();
    searchVisible.value = true;
  }
}

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  if (notebookStore.currentNote && localContent.value) {
    saveContent();
  }
});
</script>

<style scoped>
.note-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.note-header {
  padding: 12px 24px 8px;
  border-bottom: 1px solid var(--border-color);
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title-input {
  flex: 1;
  font-size: 18px;
  font-weight: 600;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-primary);
  padding: 4px 0;
}

.title-input::placeholder {
  color: var(--text-secondary);
}

.tag-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 6px;
}

.tag-badge {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 2px 8px;
  font-size: 12px;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  border-radius: 10px;
  border: 1px solid var(--border-color);
}

.tag-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  font-size: 12px;
  line-height: 1;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 50%;
  padding: 0;
}

.tag-remove:hover {
  background: var(--danger-color);
  color: white;
}

.tag-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  font-size: 14px;
  background: var(--bg-secondary);
  border: 1px dashed var(--border-color);
  border-radius: 10px;
  color: var(--text-secondary);
  cursor: pointer;
}

.tag-add:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.tag-input-wrap {
  display: inline-flex;
}

.tag-input {
  width: 80px;
  padding: 2px 8px;
  font-size: 12px;
  border: 1px solid var(--accent-color);
  border-radius: 10px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
}

.no-note-message {
  font-size: 14px;
  color: var(--text-secondary);
  font-style: italic;
}

.note-meta {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 12px;
  white-space: nowrap;
}

.editor-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.editor-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
}

.placeholder-content {
  text-align: center;
  color: var(--text-secondary);
}

.placeholder-content p {
  margin: 8px 0;
  font-size: 14px;
}

.placeholder-content .hint {
  font-size: 12px;
  opacity: 0.7;
}
</style>
