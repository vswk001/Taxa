<!-- src/components/editor/NoteEditor.vue -->
<template>
  <div class="note-editor">
    <EditorToolbar @action="handleToolbarAction" />
    <div ref="editorRef" class="editor-content"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { useNotebookStore } from '@/stores/notebook';
import { Editor, rootCtx, defaultValueCtx } from '@milkdown/kit/core';
import { commonmark } from '@milkdown/kit/preset/commonmark';
import { gfm } from '@milkdown/kit/preset/gfm';
import { history } from '@milkdown/kit/plugin/history';
import { clipboard } from '@milkdown/kit/plugin/clipboard';
import { listener, listenerCtx } from '@milkdown/kit/plugin/listener';
import EditorToolbar from './EditorToolbar.vue';

const editorRef = ref<HTMLElement>();
const notebookStore = useNotebookStore();
let editor: Editor | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(() => {
  initEditor();
});

watch(() => notebookStore.currentNote, (newNote) => {
  if (newNote && editor) {
    // Update editor content when note changes
    // The exact API may need adjustment based on Milkdown version
    try {
      editor.action((ctx: any) => {
        const editorView = ctx.get(rootCtx);
        if (editorView && editorView.state) {
          const schema = editorView.state.schema;
          const doc = schema.nodeFromJSON({
            type: 'doc',
            content: [{ type: 'paragraph', content: [{ type: 'text', text: newNote.content }] }],
          });
          const state = editorView.state.create({ doc });
          editorView.updateState(state);
        }
      });
    } catch (e) {
      console.warn('Failed to update editor content:', e);
    }
  }
});

function initEditor() {
  if (!editorRef.value) return;
  const content = notebookStore.currentNote?.content || '';

  Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, editorRef.value!);
      ctx.set(defaultValueCtx, content);
      ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
        scheduleSave(markdown);
      });
    })
    .use(commonmark)
    .use(gfm)
    .use(history)
    .use(clipboard)
    .use(listener)
    .create()
    .then((e) => { editor = e; })
    .catch((e) => {
      console.error('Failed to create Milkdown editor:', e);
    });
}

function scheduleSave(content: string) {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    if (notebookStore.currentNote) {
      notebookStore.updateNoteContent(notebookStore.currentNote.note.id, content);
    }
  }, 1000);
}

function handleToolbarAction(_action: string) {
  if (!editor) return;
  try {
    editor.action((ctx: any) => {
      const editorView = ctx.get(rootCtx);
      // Map toolbar actions to Milkdown commands
      // This will be expanded with specific command calls
      console.log('Toolbar action:', _action, 'Editor view:', editorView);
    });
  } catch (e) {
    console.warn('Failed to execute toolbar action:', e);
  }
}

onBeforeUnmount(() => {
  if (saveTimer) clearTimeout(saveTimer);
  if (editor) editor.destroy();
});
</script>

<style scoped>
.note-editor { display: flex; flex-direction: column; height: 100%; }
.editor-content {
  flex: 1; padding: 24px 32px; overflow-y: auto; font-size: 15px; line-height: 1.7;
}
.editor-content :deep(.milkdown) { outline: none; min-height: 100%; }
</style>
