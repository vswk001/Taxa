<template>
  <div class="note-editor">
    <div class="note-header">
      <input
        v-model="localTitle"
        class="title-input"
        placeholder="笔记标题..."
        @blur="saveTitle"
      />
      <span class="note-meta">{{ notebookStore.currentNote?.note.word_count || 0 }} 字</span>
    </div>
    <EditorToolbar @action="handleToolbarAction" />
    <div v-if="editorError" class="editor-error">
      <p>编辑器加载失败，使用简化模式</p>
      <textarea
        v-model="fallbackContent"
        class="fallback-editor"
        @blur="saveFallbackContent"
      />
    </div>
    <div v-else ref="editorRef" class="editor-content"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import EditorToolbar from './EditorToolbar.vue';

const notebookStore = useNotebookStore();
const editorRef = ref<HTMLElement>();
const editorError = ref(false);
const fallbackContent = ref('');
const localTitle = ref('');

// Try to import Milkdown dynamically
let editorInstance: any = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const localContent = computed({
  get: () => notebookStore.currentNote?.content || '',
  set: (val: string) => {
    if (notebookStore.currentNote) {
      notebookStore.currentNote.content = val;
    }
  }
});

onMounted(async () => {
  localTitle.value = notebookStore.currentNote?.note.title || '';
  fallbackContent.value = notebookStore.currentNote?.content || '';

  try {
    // Try to load Milkdown
    const { Editor, rootCtx, defaultValueCtx } = await import('@milkdown/kit/core');
    const { commonmark } = await import('@milkdown/kit/preset/commonmark');
    const { gfm } = await import('@milkdown/kit/preset/gfm');
    const { history } = await import('@milkdown/kit/plugin/history');
    const { clipboard } = await import('@milkdown/kit/plugin/clipboard');
    const { listener, listenerCtx } = await import('@milkdown/kit/plugin/listener');

    if (!editorRef.value) return;

    const content = localContent.value;

    editorInstance = await Editor.make()
      .config((ctx: any) => {
        ctx.set(rootCtx, editorRef.value);
        ctx.set(defaultValueCtx, content);
        ctx.get(listenerCtx).markdownUpdated((_: any, markdown: string) => {
          scheduleSave(markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(history)
      .use(clipboard)
      .use(listener)
      .create();
  } catch (e) {
    console.error('Failed to load Milkdown editor:', e);
    editorError.value = true;
  }
});

watch(() => notebookStore.currentNote, (newNote) => {
  if (newNote) {
    localTitle.value = newNote.note.title;
    fallbackContent.value = newNote.content;

    if (editorInstance && !editorError.value) {
      // Content update would be handled here for Milkdown
      // For now, we rely on the fallback content update
    }
  }
});

function scheduleSave(content: string) {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    if (notebookStore.currentNote) {
      notebookStore.updateNoteContent(notebookStore.currentNote.note.id, content);
    }
  }, 1000);
}

async function saveTitle() {
  if (notebookStore.currentNote && localTitle.value !== notebookStore.currentNote.note.title) {
    await notebookStore.updateNoteContent(notebookStore.currentNote.note.id, notebookStore.currentNote.content);
    // Update the note object title
    if (notebookStore.currentNote) {
      notebookStore.currentNote.note.title = localTitle.value;
    }
  }
}

function saveFallbackContent() {
  if (notebookStore.currentNote) {
    notebookStore.updateNoteContent(notebookStore.currentNote.note.id, fallbackContent.value);
  }
}

function handleToolbarAction(action: string) {
  if (editorError.value) {
    // Simple toolbar actions for textarea
    const textarea = document.querySelector('.fallback-editor') as HTMLTextAreaElement;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = textarea.value;
    const selected = text.substring(start, end);

    let replacement = selected;
    let cursorOffset = 0;

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
        break;
      case 'list':
        replacement = `- ${selected || '列表项'}`;
        break;
      case 'quote':
        replacement = `> ${selected || '引用内容'}`;
        break;
      case 'code':
        replacement = `\`\`\`\n${selected || '代码'}\n\`\`\``;
        break;
      case 'link':
        replacement = `[${selected || '链接文本'}](url)`;
        break;
    }

    textarea.value = text.substring(0, start) + replacement + text.substring(end);
    textarea.selectionStart = textarea.selectionEnd = start + replacement.length + cursorOffset;
    textarea.focus();
    fallbackContent.value = textarea.value;
  } else if (editorInstance) {
    // Milkdown commands would go here
    console.log('Toolbar action:', action);
  }
}

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  if (editorInstance) {
    try {
      editorInstance.destroy();
    } catch (e) {
      console.warn('Failed to destroy editor:', e);
    }
  }
});
</script>

<style scoped>
.note-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
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

.note-meta {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 12px;
}

.editor-content {
  flex: 1;
  padding: 24px 32px;
  overflow-y: auto;
  font-size: 15px;
  line-height: 1.7;
}

.editor-content :deep(.milkdown) {
  outline: none;
  min-height: 100%;
}

.editor-error {
  flex: 1;
  padding: 24px;
}

.editor-error p {
  color: var(--danger-color);
  font-size: 13px;
  margin-bottom: 12px;
}

.fallback-editor {
  width: 100%;
  height: 100%;
  min-height: 400px;
  padding: 16px;
  font-size: 15px;
  line-height: 1.7;
  font-family: inherit;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  background: var(--bg-primary);
  color: var(--text-primary);
  resize: none;
}

.fallback-editor:focus {
  outline: none;
  border-color: var(--accent-color);
}
</style>
