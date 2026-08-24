<template>
  <div class="note-editor" @keydown="handleKeyDown">
    <div v-if="notebookStore.currentNote" class="note-header">
      <div class="header-row">
        <input
          v-model="localTitle"
          class="title-input"
          :placeholder="t('editor.titlePlaceholder')"
          @blur="saveTitle"
          @keyup.enter="saveTitle"
        />
        <span class="note-meta">{{ t('editor.wordCount', { count: wordCount }) }}</span>
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
            :placeholder="t('editor.tagPlaceholder')"
            @keydown.enter.prevent="addTag"
            @keydown.escape="tagInputVisible = false"
            @blur="addTag"
          />
        </div>
        <button v-else class="tag-add" @click="showTagInput">+</button>
        <span v-if="saveError" class="save-error">{{ saveError }}</span>
      </div>
    </div>
    <div v-else class="note-header">
      <span class="no-note-message">{{ t('editor.noNote') }}</span>
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
        <p>{{ t('editor.placeholder') }}</p>
        <p class="hint">{{ t('editor.placeholderHint') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, defineAsyncComponent, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';

import EditorSearch from './EditorSearch.vue';

// Lazy-load the editor stack (Milkdown + ProseMirror + KaTeX) so the app
// shell paints before the ~1.5 MB chunk downloads and parses.
const MilkdownEditor = defineAsyncComponent(() => import('./MilkdownEditor.vue'));

const { t } = useI18n();
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
/** Transient banner shown when a save fails (the old code only logged). */
const saveError = ref('');
let saveErrorTimer: ReturnType<typeof setTimeout> | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;
let saveSeq = 0;
let isLoadingNote = false;
let isDirty = false;

// Debounced copy for word counting — running two regexes over the whole
// document on every keystroke is wasteful for large notes.
const contentForCount = ref('');
let countTimer: ReturnType<typeof setTimeout> | null = null;

const wordCount = computed(() => {
  const text = contentForCount.value.trim();
  if (!text) return 0;
  const chineseChars = text.match(/[\u4e00-\u9fa5]/g)?.length || 0;
  const englishWords = text.replace(/[\u4e00-\u9fa5]/g, ' ').match(/[a-zA-Z]+/g)?.length || 0;
  return chineseChars + englishWords;
});

function showSaveError(message: string) {
  saveError.value = message;
  if (saveErrorTimer) clearTimeout(saveErrorTimer);
  saveErrorTimer = setTimeout(() => { saveError.value = ''; }, 6000);
}

watch(() => notebookStore.currentNote, (newNote) => {
  if (newNote) {
    isLoadingNote = true;
    localTitle.value = newNote.note.title;
    localContent.value = newNote.content;
    contentForCount.value = newNote.content;
    localTags.value = [...newNote.note.tags];
    isDirty = false;
    editorStore.isModified = false;
    editorStore.openTab(newNote.note.id, newNote.note.title);
    // Let the content prop propagate to Milkdown, then clear the loading flag
    setTimeout(() => { isLoadingNote = false; }, 0);
  } else {
    isLoadingNote = false;
    localTitle.value = '';
    localContent.value = '';
    contentForCount.value = '';
    localTags.value = [];
    isDirty = false;
    editorStore.isModified = false;
  }
}, { immediate: true });

watch(() => editorStore.activeTabId, async (newTabId) => {
  // '__graph__' is a pseudo-tab, not a note id — never feed it to openNote.
  if (newTabId && newTabId !== '__graph__' && notebookStore.currentNote?.note.id !== newTabId) {
    await flushSave();
    await notebookStore.openNote(newTabId);
  }
});

watch(localContent, () => {
  if (countTimer) clearTimeout(countTimer);
  countTimer = setTimeout(() => { contentForCount.value = localContent.value; }, 300);
  if (isLoadingNote) return;
  handleContentChange();
});

function handleContentChange() {
  isDirty = true;
  editorStore.isModified = true;
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

/** Persist pending edits now. No-op when nothing changed. */
async function flushSave() {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  if (!isDirty) return;
  await saveContent();
  await editorStore.waitForSave();
}

async function saveContent() {
  if (notebookStore.currentNote && localContent.value !== undefined) {
    try {
      await notebookStore.updateNoteContent(notebookStore.currentNote.note.id, localContent.value);
      isDirty = false;
      editorStore.isModified = false;
    } catch (error) {
      console.error('Failed to save content:', error);
      showSaveError(t('editor.saveFailed'));
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
      showSaveError(t('editor.saveFailed'));
    }
  }
}

function showTagInput() {
  tagInputVisible.value = true;
  newTag.value = '';
  nextTick(() => tagInputRef.value?.focus());
}

function addTag() {
  const tag = newTag.value.trim();
  tagInputVisible.value = false;
  if (!tag || localTags.value.includes(tag)) { newTag.value = ''; return; }
  localTags.value.push(tag);
  newTag.value = '';
  saveTags();
}

function removeTag(tag: string) {
  localTags.value = localTags.value.filter(t2 => t2 !== tag);
  saveTags();
}

async function saveTags() {
  if (notebookStore.currentNote) {
    try {
      await notebookStore.updateNoteTags(notebookStore.currentNote.note.id, [...localTags.value]);
    } catch (error) {
      console.error('Failed to save tags:', error);
      showSaveError(t('editor.saveFailed'));
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

onMounted(() => {
  // Any note switch made through the store flushes pending edits first.
  notebookStore.registerPendingSave(() => flushSave());
});

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  if (saveErrorTimer) clearTimeout(saveErrorTimer);
  notebookStore.registerPendingSave(null);
  // Save even when the (cleared) content is empty — the old guard dropped
  // the final edit when a note was emptied right before navigation.
  if (notebookStore.currentNote && isDirty) {
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

.save-error {
  margin-left: auto;
  font-size: 12px;
  color: var(--danger-color);
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
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
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
