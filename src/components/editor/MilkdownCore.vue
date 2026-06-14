<template>
  <Milkdown />
</template>

<script setup lang="ts">
import { watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Milkdown, useEditor, useInstance } from '@milkdown/vue';
import { Crepe } from '@milkdown/crepe';
import '@milkdown/crepe/theme/classic.css';
import '@milkdown/crepe/theme/common/style.css';
import { EditorState } from 'prosemirror-state';
import { editorViewCtx, parserCtx } from '@milkdown/kit/core';

const { t } = useI18n();

const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

let ignoreNextUpdate = false;
let currentMarkdown = props.modelValue;

useEditor((container) => {
  const crepe = new Crepe({
    root: container,
    defaultValue: props.modelValue,
    features: {
      [Crepe.Feature.Toolbar]: true,
      [Crepe.Feature.Placeholder]: true,
      [Crepe.Feature.LinkTooltip]: true,
      [Crepe.Feature.ListItem]: true,
      [Crepe.Feature.Cursor]: true,
      [Crepe.Feature.BlockEdit]: true,
    },
    featureConfigs: {
      [Crepe.Feature.Placeholder]: {
        text: t('editor.startWriting'),
      },
    },
  });

  crepe.on((listeners) => {
    listeners.markdownUpdated((_, markdown) => {
      if (ignoreNextUpdate) return;
      currentMarkdown = markdown;
      emit('update:modelValue', markdown);
    });
  });

  return crepe;
});

const [loading, getInstance] = useInstance();

watch(() => props.modelValue, (newValue) => {
  if (newValue === currentMarkdown) return;
  const editor = getInstance();
  if (!editor || loading.value) return;

  ignoreNextUpdate = true;
  try {
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const parser = ctx.get(parserCtx);
      const doc = parser(newValue);
      if (!doc) return;
      const state = view.state;
      const tr = state.tr.replaceWith(0, state.doc.content.size, doc.content);
      tr.setMeta('addToHistory', false);
      view.dispatch(tr);
      // Reset editor state to clear undo/redo history
      view.updateState(EditorState.create({
        doc: view.state.doc,
        selection: view.state.selection,
        plugins: view.state.plugins,
      }));
    });
  } finally {
    currentMarkdown = newValue;
    setTimeout(() => { ignoreNextUpdate = false; }, 50);
  }
});

// Editor lifecycle is managed by Milkdown component's onUnmounted — no manual destroy needed
</script>

<style>
.milkdown {
  --crepe-color-background: var(--bg-primary);
  --crepe-color-on-background: var(--text-primary);
  --crepe-color-surface: var(--bg-secondary);
  --crepe-color-surface-low: var(--bg-secondary);
  --crepe-color-on-surface: var(--text-primary);
  --crepe-color-on-surface-variant: var(--text-secondary);
  --crepe-color-outline: var(--border-color);
  --crepe-color-primary: var(--accent-color);
  --crepe-color-secondary: var(--bg-secondary);
  --crepe-color-on-secondary: var(--text-primary);
  --crepe-color-inverse: var(--text-primary);
  --crepe-color-on-inverse: var(--bg-primary);
  --crepe-color-inline-code: var(--accent-color);
  --crepe-color-error: var(--danger-color);
  --crepe-color-hover: var(--bg-secondary);
  --crepe-color-selected: var(--bg-secondary);
  --crepe-color-inline-area: var(--bg-secondary);
  --crepe-font-default: var(--font-sans);
  --crepe-font-code: var(--font-mono);

  height: 100%;
}

.milkdown .ProseMirror {
  padding: 24px 32px !important;
  font-size: 15px;
  line-height: 1.7;
}

.milkdown .ProseMirror p {
  font-size: 15px;
  line-height: 1.7;
}

.milkdown .ProseMirror h1,
.milkdown .ProseMirror h2,
.milkdown .ProseMirror h3,
.milkdown .ProseMirror h4,
.milkdown .ProseMirror h5,
.milkdown .ProseMirror h6 {
  font-family: var(--font-sans) !important;
  font-weight: 600 !important;
}

.milkdown .ProseMirror h1 { font-size: 1.8em !important; line-height: 1.3 !important; }
.milkdown .ProseMirror h2 { font-size: 1.4em !important; line-height: 1.3 !important; }
.milkdown .ProseMirror h3 { font-size: 1.2em !important; line-height: 1.4 !important; }
.milkdown .ProseMirror h4 { font-size: 1.1em !important; line-height: 1.4 !important; }

.milkdown .ProseMirror code {
  background: var(--bg-secondary);
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.9em;
  font-family: var(--font-mono);
}

.milkdown .ProseMirror pre {
  background: var(--bg-secondary);
  padding: 12px 16px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.milkdown .ProseMirror pre code {
  background: transparent;
  padding: 0;
}

.milkdown .ProseMirror blockquote {
  border-left: 3px solid var(--border-color);
  padding-left: 16px;
  color: var(--text-secondary);
  margin: 0.5em 0;
}

.milkdown .ProseMirror ul,
.milkdown .ProseMirror ol {
  padding-left: 24px;
  margin: 0.5em 0;
}

.milkdown .ProseMirror img {
  max-width: 100%;
  border-radius: 6px;
}

.milkdown .ProseMirror a {
  color: var(--accent-color);
  text-decoration: none;
}

.milkdown .ProseMirror a:hover {
  text-decoration: underline;
}

.milkdown .ProseMirror hr {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 1em 0;
}

.milkdown .ProseMirror table {
  border-collapse: collapse;
  width: 100%;
  margin: 0.8em 0;
}

.milkdown .ProseMirror th,
.milkdown .ProseMirror td {
  border: 1px solid var(--border-color);
  padding: 8px 12px;
  text-align: left;
}

.milkdown .ProseMirror th {
  background: var(--bg-secondary);
  font-weight: 600;
}
</style>
