<template>
  <div class="tree-node">
    <div
      class="node-row folder-row"
      :class="{ active: isSelected }"
      @click="handleClick"
      @contextmenu.prevent="emit('contextmenu-folder', $event, folder)"
    >
      <span class="toggle" @click.stop="toggleExpanded">
        <span v-if="hasChildren || folderNotes.length > 0">{{ expanded ? '▼' : '▶' }}</span>
        <span v-else class="no-toggle"></span>
      </span>
      <span class="icon">{{ expanded ? '📂' : '📁' }}</span>
      <span class="name">{{ folder.name }}</span>
      <span v-if="folder.note_count > 0" class="count">{{ folder.note_count }}</span>
    </div>

    <div v-if="expanded" class="children">
      <TreeNode
        v-for="child in folder.children"
        :key="child.path"
        :folder="child"
        :selected-path="selectedPath"
        :selected-note-id="selectedNoteId"
        :note-map="noteMap"
        @select="emit('select', $event)"
        @select-note="emit('select-note', $event)"
        @contextmenu-folder="(e: MouseEvent, f: Folder) => emit('contextmenu-folder', e, f)"
        @contextmenu-note="(e: MouseEvent, n: Note) => emit('contextmenu-note', e, n)"
      />

      <div
        v-for="note in folderNotes"
        :key="note.id"
        class="note-row"
        :class="{ active: selectedNoteId === note.id }"
        @click="emit('select-note', note)"
        @contextmenu.prevent="emit('contextmenu-note', $event, note)"
      >
        <span class="note-icon">📄</span>
        <span class="note-title">{{ note.title }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Folder, Note } from '@/types/notebook';

const props = defineProps<{
  folder: Folder;
  selectedPath: string;
  selectedNoteId: string | null;
  noteMap: Map<string, Note[]>;
}>();

const emit = defineEmits<{
  select: [path: string];
  'select-note': [note: Note];
  'contextmenu-folder': [event: MouseEvent, folder: Folder];
  'contextmenu-note': [event: MouseEvent, note: Note];
}>();

const expanded = ref(false);

const hasChildren = computed(() => props.folder.children && props.folder.children.length > 0);

const folderNotes = computed(() => props.noteMap.get(props.folder.path) || []);

const isSelected = computed(() => props.selectedPath === props.folder.path);

function toggleExpanded() {
  expanded.value = !expanded.value;
}

function handleClick() {
  expanded.value = !expanded.value;
  emit('select', props.folder.path);
}
</script>

<style scoped>
.tree-node { user-select: none; }

.node-row {
  display: flex; align-items: center; gap: 4px;
  padding: 5px 8px; cursor: pointer; border-radius: 4px; font-size: 13px;
}
.node-row:hover { background: var(--border-color); }
.node-row.active { background: var(--accent-color); color: white; }

.toggle {
  width: 16px; text-align: center; font-size: 10px;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer;
}
.no-toggle { display: inline-block; width: 100%; }
.icon { font-size: 14px; }
.name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.count {
  font-size: 11px; color: var(--text-secondary);
  background: var(--border-color); padding: 1px 6px; border-radius: 10px;
  min-width: 20px; text-align: center;
}
.node-row.active .count { background: rgba(255,255,255,0.2); color: white; }

.children { padding-left: 16px; }

.note-row {
  display: flex; align-items: center; gap: 4px;
  padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 13px;
}
.note-row:hover { background: var(--border-color); }
.note-row.active { background: var(--accent-color); color: white; }
.note-icon { font-size: 13px; }
.note-title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
