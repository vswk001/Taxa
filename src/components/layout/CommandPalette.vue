<template>
  <Teleport to="body">
    <div v-if="visible" class="palette-overlay" @click.self="emit('close')">
      <div class="palette">
        <input
          ref="inputRef"
          v-model="query"
          class="palette-input"
          :placeholder="t('palette.placeholder')"
          @keydown.down.prevent="move(1)"
          @keydown.up.prevent="move(-1)"
          @keydown.enter.prevent="runSelected"
          @keydown.escape="emit('close')"
        />
        <div class="palette-results">
          <button
            v-for="(item, i) in results"
            :key="item.kind + item.id"
            class="palette-item"
            :class="{ selected: i === selected }"
            @mouseenter="selected = i"
            @click="activate(item)"
          >
            <span class="item-icon">{{ item.icon }}</span>
            <span class="item-label">{{ item.label }}</span>
            <span class="item-kind">{{ t('palette.kind.' + item.kind) }}</span>
          </button>
          <div v-if="!results.length" class="palette-empty">{{ t('palette.empty') }}</div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNotebookStore } from '@/stores/notebook';
import { useEditorStore } from '@/stores/editor';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: []; 'run-action': [id: string]; 'open-search': [] }>();
const { t } = useI18n();
const notebookStore = useNotebookStore();
const editorStore = useEditorStore();

interface PaletteItem {
  kind: 'note' | 'folder' | 'action';
  id: string;
  label: string;
  icon: string;
}

const query = ref('');
const selected = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

const ACTIONS: PaletteItem[] = [
  { kind: 'action', id: 'new-note', label: '', icon: '📝' },
  { kind: 'action', id: 'daily', label: '', icon: '📅' },
  { kind: 'action', id: 'search', label: '', icon: '🔍' },
  { kind: 'action', id: 'graph', label: '', icon: '🔗' },
  { kind: 'action', id: 'tags', label: '', icon: '🏷' },
  { kind: 'action', id: 'trash', label: '', icon: '🗑' },
  { kind: 'action', id: 'settings', label: '', icon: '⚙️' },
];

const results = computed<PaletteItem[]>(() => {
  const q = query.value.trim().toLowerCase();
  const notes: PaletteItem[] = notebookStore.notes
    .filter((n) => !q || n.title.toLowerCase().includes(q))
    .slice(0, q ? 8 : 6)
    .map((n) => ({ kind: 'note', id: n.id, label: n.title, icon: '📄' }));
  const folders: PaletteItem[] = notebookStore.folders
    .flatMap((f) => [f, ...f.children])
    .filter((f) => q && f.name.toLowerCase().includes(q))
    .slice(0, 4)
    .map((f) => ({ kind: 'folder', id: f.path, label: f.name, icon: '📁' }));
  const actions = ACTIONS.map((a) => ({ ...a, label: t('palette.action.' + a.id) }))
    .filter((a) => !q || a.label.toLowerCase().includes(q));
  return [...notes, ...folders, ...actions];
});

watch(results, () => { selected.value = 0; });

watch(
  () => props.visible,
  async (v) => {
    if (v) {
      query.value = '';
      selected.value = 0;
      await nextTick();
      inputRef.value?.focus();
    }
  },
);

function move(delta: number) {
  if (!results.value.length) return;
  selected.value = (selected.value + delta + results.value.length) % results.value.length;
}

function runSelected() {
  const item = results.value[selected.value];
  if (item) activate(item);
}

async function activate(item: PaletteItem) {
  emit('close');
  if (item.kind === 'note') {
    await notebookStore.openNote(item.id);
    const note = notebookStore.currentNote;
    if (note) editorStore.openTab(note.note.id, note.note.title);
  } else if (item.kind === 'folder') {
    notebookStore.currentFolder = item.id;
    notebookStore.viewMode = 'folder';
    notebookStore.selectedFolderForList = item.id;
  } else {
    if (item.id === 'search') emit('open-search');
    else emit('run-action', item.id);
  }
}
</script>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 180;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 12vh;
}

.palette {
  width: 520px;
  max-height: 420px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.palette-input {
  border: none;
  border-bottom: 1px solid var(--border-color);
  outline: none;
  padding: 14px 16px;
  font-size: 15px;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.palette-results {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 12px;
  background: none;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
}

.palette-item.selected {
  background: var(--bg-secondary);
}

.item-icon {
  font-size: 14px;
}

.item-label {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-kind {
  font-size: 11px;
  color: var(--text-secondary);
}

.palette-empty {
  padding: 24px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}
</style>
