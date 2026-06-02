<!-- src/components/tree/TreeNode.vue -->
<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ active: isSelected }"
      @click="handleClick"
      @contextmenu.prevent="emit('contextmenu', $event, folder)"
    >
      <span class="toggle" @click.stop="toggleExpanded">
        <span v-if="folder.children.length > 0">{{ expanded ? '▼' : '▶' }}</span>
      </span>
      <span class="icon">📁</span>
      <span class="name">{{ folder.name }}</span>
      <span class="count">{{ folder.note_count }}</span>
    </div>
    <div v-if="expanded" class="children">
      <TreeNode
        v-for="child in folder.children"
        :key="child.path"
        :folder="child"
        :selected-path="selectedPath"
        @select="emit('select', $event)"
        @contextmenu="emit('contextmenu', $event, folder)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Folder } from '@/types/notebook';

const props = defineProps<{
  folder: Folder;
  selectedPath: string;
}>();

const emit = defineEmits<{
  select: [path: string];
  contextmenu: [event: MouseEvent, folder: Folder];
}>();

const expanded = ref(true);
const isSelected = computed(() => props.selectedPath === props.folder.path);

function toggleExpanded() { expanded.value = !expanded.value; }
function handleClick() {
  expanded.value = true;
  emit('select', props.folder.path);
}
</script>

<style scoped>
.tree-node { user-select: none; }
.node-row {
  display: flex; align-items: center; gap: 4px;
  padding: 4px 8px; cursor: pointer; border-radius: 4px; font-size: 13px;
}
.node-row:hover { background: var(--border-color); }
.node-row.active { background: var(--accent-color); color: white; }
.toggle { width: 16px; text-align: center; font-size: 10px; }
.icon { font-size: 14px; }
.name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.count { font-size: 11px; color: var(--text-secondary); }
.children { padding-left: 16px; }
</style>
