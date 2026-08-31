<template>
  <div ref="containerRef" class="milkdown-container" @paste="onPaste" @drop.prevent="onDrop" @mouseup="updateSelectionPanel" @keyup="updateSelectionPanel">
    <Milkdown />
    <SelectionAiPanel
      :visible="selectionAi.visible"
      :x="selectionAi.x"
      :y="selectionAi.y"
      :state="selectionAi.state"
      :result="selectionAi.result"
      :error="selectionAi.error"
      @action="runSelectionAction"
      @apply="applySelectionResult"
      @cancel="hideSelectionPanel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { Milkdown, useEditor, useInstance } from '@milkdown/vue';
import { Crepe } from '@milkdown/crepe';
import '@milkdown/crepe/theme/classic.css';
import '@milkdown/crepe/theme/common/style.css';
import { EditorState } from 'prosemirror-state';
import { editorViewCtx, parserCtx } from '@milkdown/kit/core';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import SelectionAiPanel from './SelectionAiPanel.vue';

const { t, locale } = useI18n();

const props = defineProps<{ modelValue: string; noteId?: string }>();
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
const containerRef = ref<HTMLElement | null>(null);

// ---- selection AI ---------------------------------------------------------
// A non-empty selection in an open note shows a floating action menu; the
// recorded ProseMirror range lets the result replace/extend exactly the
// selected text even after the panel steals focus.
const selectionAi = ref({
  visible: false,
  x: 0,
  y: 0,
  state: 'idle' as 'idle' | 'loading' | 'done' | 'error',
  result: '',
  error: '',
});
let selectionRange: { from: number; to: number; text: string } | null = null;

function updateSelectionPanel() {
  const editor = getInstance();
  if (!editor || loading.value || !props.noteId) {
    selectionAi.value.visible = false;
    selectionRange = null;
    return;
  }
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { from, to } = view.state.selection;
    const text = view.state.doc.textBetween(from, to, '\n');
    if (to <= from || !text.trim() || text.length > 6000) {
      selectionAi.value.visible = false;
      selectionRange = null;
      return;
    }
    selectionRange = { from, to, text };
    try {
      const coords = view.coordsAtPos(to);
      selectionAi.value = {
        visible: true,
        x: coords.left,
        y: coords.bottom + 8,
        state: 'idle',
        result: '',
        error: '',
      };
    } catch {
      selectionAi.value.visible = false;
    }
  });
}

async function runSelectionAction(action: string) {
  if (!selectionRange) return;
  selectionAi.value.state = 'loading';
  try {
    const result = await invoke<string>('ai_text_action', {
      text: selectionRange.text,
      action,
      locale: locale.value,
    });
    selectionAi.value.result = result || '';
    selectionAi.value.state = 'done';
  } catch (e) {
    selectionAi.value.error = e instanceof Error ? e.message : String(e);
    selectionAi.value.state = 'error';
  }
}

function applySelectionResult(mode: 'replace' | 'insert') {
  const range = selectionRange;
  if (!range) return;
  const editor = getInstance();
  if (editor && !loading.value) {
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const text = selectionAi.value.result;
      const tr = mode === 'replace'
        ? view.state.tr.insertText(text, range.from, range.to)
        : view.state.tr.insertText(text, range.to);
      view.dispatch(tr.scrollIntoView());
      view.focus();
    });
  }
  selectionAi.value.visible = false;
  selectionRange = null;
}

function hideSelectionPanel() {
  selectionAi.value.visible = false;
  selectionRange = null;
}

// Programmatic-set echo suppression: instead of a time-based flag (which
// swallowed keystrokes within its window), the first markdownUpdated after a
// programmatic set is ignored only when it echoes the value we applied.
let applyingSetValue = false;
let lastAppliedValue = props.modelValue;
let currentMarkdown = props.modelValue;
// Values arriving while the editor is still initializing are queued and
// applied once it is ready (previously they were silently dropped).
let pendingValue: string | null = null;

// ---- image attachments ---------------------------------------------------
// Markdown stores portable relative paths (attachments/x.png); the webview
// can only load local files through the asset protocol. Rewrite <img src>
// for display, and undo the rewrite when emitting markdown back.

let assetPrefix = ''; // convertFileSrc("<attachments dir>/")

void (async () => {
  try {
    const info = await invoke<{ attachments_dir: string }>('get_vault_info');
    assetPrefix = convertFileSrc(`${info.attachments_dir.split('\\').join('/')}/`);
  } catch {
    /* images stay as plain relative paths */
  }
})();

function toDisplayMarkdown(markdown: string): string {
  return assetPrefix ? markdown.split(assetPrefix).join('attachments/') : markdown;
}

let observer: MutationObserver | null = null;

function rewriteImgSrc(root: HTMLElement) {
  if (!assetPrefix) return;
  for (const img of root.querySelectorAll<HTMLImageElement>('img[src^="attachments/"]')) {
    img.src = assetPrefix + img.getAttribute('src');
  }
}

onMounted(() => {
  observer = new MutationObserver(() => {
    if (containerRef.value) rewriteImgSrc(containerRef.value);
  });
  if (containerRef.value) {
    observer.observe(containerRef.value, { childList: true, subtree: true, attributes: true, attributeFilter: ['src'] });
    rewriteImgSrc(containerRef.value);
  }
});

onBeforeUnmount(() => observer?.disconnect());

/** Insert an image by markdown path. The markdown is parsed first so the
 *  document receives a real image node — inserting the raw "![](path)"
 *  string as text leaves the path visible until the next full re-parse. */
function insertImage(path: string) {
  const editor = getInstance();
  if (!editor || loading.value) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const parser = ctx.get(parserCtx);
    const doc = parser(`![](${path})`);
    const img = doc?.content.firstChild?.firstChild;
    if (!img || img.type.name !== 'image') return;
    view.dispatch(view.state.tr.replaceSelectionWith(img).scrollIntoView());
    view.focus();
  });
}

async function saveAndInsert(file: File) {
  if (!props.noteId) return;
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  const base64 = btoa(binary);
  try {
    const path = await invoke<string>('save_attachment', {
      fileName: file.name || 'image.png',
      data: base64,
    });
    insertImage(path);
  } catch (e) {
    console.error('failed to save attachment:', e);
  }
}

function onPaste(e: ClipboardEvent) {
  const files = Array.from(e.clipboardData?.files ?? []).filter((f) => f.type.startsWith('image/'));
  if (!files.length || !props.noteId) return;
  e.preventDefault();
  for (const file of files) void saveAndInsert(file);
}

async function onDrop(e: DragEvent) {
  const files = Array.from(e.dataTransfer?.files ?? []).filter((f) => f.type.startsWith('image/'));
  if (!files.length || !props.noteId) return;
  for (const file of files) await saveAndInsert(file);
}

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
      if (applyingSetValue && markdown === lastAppliedValue) return; // our own echo
      applyingSetValue = false;
      const display = toDisplayMarkdown(markdown);
      currentMarkdown = display;
      emit('update:modelValue', display);
    });
  });

  return crepe;
});

const [loading, getInstance] = useInstance();

function applyValue(newValue: string) {
  const editor = getInstance();
  if (!editor) {
    pendingValue = newValue;
    return;
  }
  applyingSetValue = true;
  lastAppliedValue = newValue;
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
  }
}

watch(() => props.modelValue, (newValue) => {
  if (newValue === currentMarkdown) return;
  if (loading.value) {
    pendingValue = newValue;
    return;
  }
  applyValue(newValue);
});

watch(loading, (isLoading) => {
  if (!isLoading && pendingValue !== null && pendingValue !== currentMarkdown) {
    const value = pendingValue;
    pendingValue = null;
    applyValue(value);
  }
});

// Editor lifecycle is managed by Milkdown component's onUnmounted — no manual destroy needed
</script>

<style>
.milkdown-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.milkdown-container .ProseMirror img {
  max-width: 100%;
}

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
