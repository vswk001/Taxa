<template>
  <Teleport to="body">
    <div v-if="visible" class="link-suggest" :style="{ left: x + 'px', top: y + 'px' }">
      <button
        v-for="(title, i) in items"
        :key="title"
        class="link-item"
        :class="{ selected: i === selected }"
        @mousedown.prevent="$emit('pick', title)"
        @mouseenter="$emit('hover', i)"
      >{{ title }}</button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean; x: number; y: number; items: string[]; selected: number }>();
defineEmits<{ pick: [title: string]; hover: [index: number] }>();
</script>

<style scoped>
.link-suggest {
  position: fixed;
  z-index: 290;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
  min-width: 180px;
  max-width: 320px;
  max-height: 220px;
  overflow-y: auto;
  padding: 4px;
}

.link-item {
  display: block;
  width: 100%;
  padding: 6px 10px;
  font-size: 13px;
  text-align: left;
  background: none;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-item.selected {
  background: var(--bg-secondary);
  color: var(--accent-color);
}
</style>
