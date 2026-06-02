<!-- src/components/layout/MenuBar.vue -->
<template>
  <div class="menu-bar">
    <div class="menu-left">
      <span class="app-title">Taxis</span>
      <button v-for="menu in menus" :key="menu.label" class="menu-item">
        {{ menu.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
const menus = [
  { label: '文件' },
  { label: '编辑' },
  { label: '视图' },
  { label: '工具' },
  { label: '帮助' },
];
</script>

<style scoped>
.menu-bar {
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  background: var(--bg-sidebar);
  border-bottom: 1px solid var(--border-color);
  -webkit-app-region: drag;
  user-select: none;
}
.menu-left { display: flex; align-items: center; gap: 2px; -webkit-app-region: no-drag; }
.app-title { font-weight: 600; margin-right: 16px; font-size: 13px; color: var(--text-primary); }
.menu-item {
  padding: 4px 10px;
  font-size: 13px;
  color: var(--text-secondary);
  border-radius: 4px;
}
.menu-item:hover { background: var(--border-color); color: var(--text-primary); }
</style>
