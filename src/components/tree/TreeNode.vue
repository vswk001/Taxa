<template>
  <div class="tree-node">
    <!-- Folder Node -->
    <div
      class="node-row folder-row"
      :class="{ active: isSelected, expanded }"
      @click="handleClick"
      @contextmenu.prevent="handleContextMenu"
    >
      <span class="toggle" @click.stop="toggleExpanded">
        <span v-if="hasChildren">{{ expanded ? '▼' : '▶' }}</span>
        <span v-else class="no-toggle"></span>
      </span>
      <span class="icon">{{ expanded ? '📂' : '📁' }}</span>
      <span class="name">{{ folder.name }}</span>
      <span v-if="folder.note_count > 0" class="count">{{ folder.note_count }}</span>
    </div>

    <!-- Children (subfolders) -->
    <div v-if="expanded && hasChildren" class="children">
      <TreeNode
        v-for="child in folder.children"
        :key="child.path"
        :folder="child"
        :selected-path="selectedPath"
        :note-map="noteMap"
        @select="emit('select', $event)"
        @select-note="emit('select-note', $event)"
        @contextmenu-folder="(e, f) => emit('contextmenu-folder', e, f)"
        @contextmenu-note="(e, n) => emit('contextmenu-note', e, n)"
      />
    </div>

    <!-- Notes in this folder -->
    <div v-if="expanded && folderNotes.length > 0" class="folder-notes">
      <div
        v-for="note in folderNotes"
        :key="note.id"
        class="note-row"
        :class="{ active: selectedNoteId === note.id }"
        @click="handleNoteClick(note)"
        @contextmenu.prevent="handleNoteContextMenu($event, note)"
      >
        <span class="note-toggle"></span>
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
  noteMap: Map<string, Note[]>;
}>();

const emit = defineEmits<{
  select: [path: string];
  'select-note': [note: Note];
  'contextmenu-folder': [event: MouseEvent, folder: Folder];
  'contextmenu-note': [event: MouseEvent, note: Note];
  contextmenu: [event: MouseEvent, item: Folder | Note];
}>();

const expanded = ref(false);

const hasChildren = computed(() => props.folder.children && props.folder.children.length > 0);

const folderNotes = computed(() => {
  return props.noteMap.get(props.folder.path) || [];
});

const isSelected = computed(() => props.selectedPath === props.folder.path);

// Check if this is the currently selected note
const selectedNoteId = computed(() => {
  // This would be passed from parent in a real implementation
  // For now, we'll rely on the active class from the store
  return null;
});

function toggleExpanded() {
  expanded.value = !expanded.value;
}

function handleClick() {
  expanded.value = true;
  emit('select', props.folder.path);
}

function handleNoteClick(note: Note) {
  emit('select-note', note);
}

function handleContextMenu(event: MouseEvent) {
  emit('contextmenu-folder', event, props.folder);
}

function handleNoteContextMenu(event: MouseEvent, note: Note) {
  emit('contextmenu-note', event, note);
}
</script>

<style scoped>
.tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  transition: background 0.15s ease;
}

.node-row:hover {
  background: var(--border-color);
}

.node-row.active {
  background: var(--accent-color);
  color: white;
}

.toggle {
  width: 16px;
  text-align: center;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.no-toggle {
  display: inline-block;
  width: 100%;
}

.icon {
  font-size: 14px;
}

.name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.count {
  font-size: 11px;
  color: var(--text-secondary);
  background: var(--border-color);
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
}

.node-row.active .count {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.children {
  padding-left: 16px;
}

.folder-notes {
  padding-left: 20px;
}

.note-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  transition: background 0.15s ease;
}

.note-row:hover {
  background: var(--border-color);
}

.note-row.active {
  background: var(--accent-color);
  color: white;
}

.note-toggle {
  width: 16px;
  display: inline-block;
}

.note-icon {
  font-size: 13px;
}

.note-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
