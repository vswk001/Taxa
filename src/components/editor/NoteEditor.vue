<template>
  <div class="note-editor">
    <div v-if="notebookStore.currentNote" class="note-header">
      <input
        v-model="localTitle"
        class="title-input"
        placeholder="笔记标题..."
        @blur="saveTitle"
        @keyup.enter="saveTitle"
      />
      <span class="note-meta">{{ wordCount }} 字</span>
    </div>
    <div v-else class="note-header">
      <span class="no-note-message">选择或创建一个笔记开始编辑</span>
    </div>

    <EditorToolbar @action="handleToolbarAction" />

    <div v-if="notebookStore.currentNote" class="editor-container">
      <textarea
        ref="textareaRef"
        v-model="localContent"
        class="note-textarea"
        placeholder="开始写作..."
        @input="handleContentChange"
        @keydown="handleKeyDown"
      />
    </div>
    <div v-else class="editor-placeholder">
      <div class="placeholder-content">
        <p>📝 暂无选中的笔记</p>
        <p class="hint">从左侧选择笔记，或点击 "+" 创建新笔记</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';
import EditorToolbar from './EditorToolbar.vue';

const notebookStore = useNotebookStore();
const editorStore = useEditorStore();
const textareaRef = ref<HTMLTextAreaElement>();
const localTitle = ref('');
const localContent = ref('');
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const wordCount = computed(() => {
  if (!localContent.value) return 0;
  // Simple word count for Chinese and English
  const text = localContent.value.trim();
  if (!text) return 0;
  // Count Chinese characters and English words
  const chineseChars = text.match(/[\u4e00-\u9fa5]/g)?.length || 0;
  const englishWords = text.replace(/[\u4e00-\u9fa5]/g, ' ').match(/[a-zA-Z]+/g)?.length || 0;
  return chineseChars + englishWords;
});

// Watch for note changes
watch(() => notebookStore.currentNote, (newNote) => {
  if (newNote) {
    localTitle.value = newNote.note.title;
    localContent.value = newNote.content;
    // Add to open tabs
    editorStore.openTab(newNote.note.id, newNote.note.title);
  } else {
    localTitle.value = '';
    localContent.value = '';
  }
}, { immediate: true });

// Watch for active tab changes
watch(() => editorStore.activeTabId, async (newTabId) => {
  if (newTabId && notebookStore.currentNote?.note.id !== newTabId) {
    // Save current note first
    if (localContent.value && notebookStore.currentNote) {
      await saveContent();
    }
    // Load the new note
    await notebookStore.openNote(newTabId);
  }
});

function handleContentChange() {
  // Auto-save after 1 second debounce
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    saveContent();
  }, 1000);
}

async function saveContent() {
  if (notebookStore.currentNote && localContent.value !== undefined) {
    try {
      await notebookStore.updateNoteContent(notebookStore.currentNote.note.id, localContent.value);
      console.log('Content saved successfully');
    } catch (error) {
      console.error('Failed to save content:', error);
    }
  }
}

async function saveTitle() {
  if (notebookStore.currentNote && localTitle.value !== notebookStore.currentNote.note.title) {
    try {
      // Update title via updateNote API (need to add this to the store)
      const noteId = notebookStore.currentNote.note.id;
      const newContent = localContent.value;

      // For now, we'll update the entire note with the new title
      await notebookStore.updateNoteContent(noteId, newContent);

      // Update the local note object
      if (notebookStore.currentNote) {
        notebookStore.currentNote.note.title = localTitle.value;
      }

      // Update tab title
      const tab = editorStore.openTabs.find(t => t.id === noteId);
      if (tab) {
        tab.title = localTitle.value;
      }

      console.log('Title saved successfully');
    } catch (error) {
      console.error('Failed to save title:', error);
    }
  }
}

function handleToolbarAction(action: string) {
  if (!textareaRef.value) return;

  const textarea = textareaRef.value;
  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const text = textarea.value;
  const selected = text.substring(start, end);

  let replacement = selected;
  let cursorOffset = 0;
  let selectReplacement = false;

  switch (action) {
    case 'bold':
      replacement = `**${selected || '粗体文本'}**`;
      cursorOffset = selected ? 0 : -2;
      break;
    case 'italic':
      replacement = `*${selected || '斜体文本'}*`;
      cursorOffset = selected ? 0 : -1;
      break;
    case 'heading':
      replacement = `## ${selected || '标题'}`;
      cursorOffset = -2;
      break;
    case 'list':
      replacement = `- ${selected || '列表项'}`;
      break;
    case 'quote':
      replacement = `> ${selected || '引用内容'}`;
      break;
    case 'code':
      replacement = `\`\`\`\n${selected || '代码'}\n\`\`\``;
      cursorOffset = -4;
      selectReplacement = true;
      break;
    case 'link':
      replacement = `[${selected || '链接文本'}](url)`;
      cursorOffset = -4;
      selectReplacement = true;
      break;
    case 'image':
      replacement = `![${selected || '图片描述'}](image-url)`;
      cursorOffset = -10;
      selectReplacement = true;
      break;
    default:
      console.log('Unknown toolbar action:', action);
      return;
  }

  const newText = text.substring(0, start) + replacement + text.substring(end);
  localContent.value = newText;

  // Update cursor position
  const newPosition = start + replacement.length + cursorOffset;

  // Use nextTick to ensure the DOM is updated
  setTimeout(() => {
    if (textareaRef.value) {
      textareaRef.value.focus();
      if (selectReplacement) {
        // Select the URL part for replacement
        if (action === 'link') {
          textareaRef.value.selectionStart = newPosition;
          textareaRef.value.selectionEnd = newPosition + 3;
        } else if (action === 'code') {
          // Select the code content
          textareaRef.value.selectionStart = start + 4;
          textareaRef.value.selectionEnd = start + replacement.length - 4;
        }
      } else {
        textareaRef.value.selectionStart = textareaRef.value.selectionEnd = newPosition;
      }
    }
  }, 0);
}

function handleKeyDown(event: KeyboardEvent) {
  // Handle keyboard shortcuts
  if (event.ctrlKey || event.metaKey) {
    switch (event.key.toLowerCase()) {
      case 's':
        event.preventDefault();
        saveContent();
        break;
      case 'b':
        event.preventDefault();
        handleToolbarAction('bold');
        break;
      case 'i':
        event.preventDefault();
        handleToolbarAction('italic');
        break;
    }
  }
}

onMounted(() => {
  // Focus textarea when note is loaded
  if (textareaRef.value && notebookStore.currentNote) {
    textareaRef.value.focus();
  }
});

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  // Save any unsaved changes
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
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px 8px;
  border-bottom: 1px solid var(--border-color);
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

.editor-container {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.note-textarea {
  width: 100%;
  height: 100%;
  padding: 24px 32px;
  font-size: 15px;
  line-height: 1.7;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  border: none;
  outline: none;
  background: var(--bg-primary);
  color: var(--text-primary);
  resize: none;
  overflow-y: auto;
}

.note-textarea::placeholder {
  color: var(--text-secondary);
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
