<template>
  <div class="menu-bar">
    <div class="menu-left">
      <span class="app-title">Taxa</span>
      <div v-for="menu in menus" :key="menu.label" class="menu-wrapper" @mouseenter="handleMouseEnter(menu.label)">
        <button class="menu-item" @click="toggleMenu(menu.label)">
          {{ menu.label }}
        </button>
        <div v-if="activeMenu === menu.label" class="dropdown-menu" @mouseleave="closeMenu">
          <template v-for="item in menu.items" :key="item.label || 'separator'">
            <div v-if="item.separator" class="menu-separator"></div>
            <div v-else-if="item.submenu" class="dropdown-item has-submenu" @mouseenter="openSubmenu = item.label ?? null" @mouseleave="openSubmenu = null">
              <span>{{ item.label }}</span>
              <span class="submenu-arrow">▸</span>
              <div v-if="openSubmenu === item.label" class="submenu">
                <button v-for="sub in item.submenu" :key="sub.label" class="dropdown-item" @click="handleAction(sub.action)">
                  {{ sub.label }}
                </button>
              </div>
            </div>
            <button v-else class="dropdown-item" @click="handleAction(item.action)">
              {{ item.label }}
              <span v-if="item.shortcut" class="shortcut">{{ item.shortcut }}</span>
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{
  openSettings: [];
  toggleSearch: [];
  toggleGraph: [];
  toggleSidebar: [];
  importFile: [];
  importFolder: [];
  exportFile: [];
  exportFolder: [];
}>();

const activeMenu = ref<string | null>(null);
const openSubmenu = ref<string | null>(null);

interface MenuItem {
  label?: string;
  action?: string;
  shortcut?: string;
  separator?: boolean;
  submenu?: { label: string; action: string }[];
}

interface Menu {
  label: string;
  items: MenuItem[];
}

const menus: Menu[] = [
  {
    label: '文件',
    items: [
      {
        label: '打开', submenu: [
          { label: '打开文件...', action: 'importFile' },
          { label: '打开目录...', action: 'importFolder' },
        ]
      },
      {
        label: '导出', submenu: [
          { label: '导出为文件...', action: 'exportFile' },
          { label: '导出为目录...', action: 'exportFolder' },
        ]
      },
      { separator: true },
      { label: '设置', action: 'openSettings', shortcut: 'Ctrl+,' },
    ]
  },
  {
    label: '编辑',
    items: [
      { label: '撤销', action: 'undo', shortcut: 'Ctrl+Z' },
      { label: '重做', action: 'redo', shortcut: 'Ctrl+Y' },
      { separator: true },
      { label: '剪切', action: 'cut', shortcut: 'Ctrl+X' },
      { label: '复制', action: 'copy', shortcut: 'Ctrl+C' },
      { label: '粘贴', action: 'paste', shortcut: 'Ctrl+V' },
    ]
  },
  {
    label: '视图',
    items: [
      { label: '切换搜索面板', action: 'toggleSearch', shortcut: 'Ctrl+K' },
      { label: '切换图谱视图', action: 'toggleGraph', shortcut: 'Ctrl+G' },
      { label: '切换侧边栏', action: 'toggleSidebar' },
    ]
  },
  {
    label: '工具',
    items: [
      { label: 'AI 助手', action: 'toggleAi' },
      { label: '设置', action: 'openSettings' },
    ]
  },
  {
    label: '帮助',
    items: [
      { label: '关于 Taxa', action: 'about' },
    ]
  },
];

function toggleMenu(label: string) {
  if (activeMenu.value === label) {
    activeMenu.value = null;
  } else {
    activeMenu.value = label;
    openSubmenu.value = null;
  }
}

function handleMouseEnter(label: string) {
  if (activeMenu.value) {
    activeMenu.value = label;
    openSubmenu.value = null;
  }
}

function closeMenu() {
  activeMenu.value = null;
  openSubmenu.value = null;
}

function handleAction(action?: string) {
  closeMenu();
  if (!action) return;

  switch (action) {
    case 'openSettings':
      emit('openSettings');
      break;
    case 'toggleSearch':
      emit('toggleSearch');
      break;
    case 'toggleGraph':
      emit('toggleGraph');
      break;
    case 'toggleSidebar':
      emit('toggleSidebar');
      break;
    case 'importFile':
      emit('importFile');
      break;
    case 'importFolder':
      emit('importFolder');
      break;
    case 'exportFile':
      emit('exportFile');
      break;
    case 'exportFolder':
      emit('exportFolder');
      break;
    case 'toggleAi':
      console.log('Toggle AI');
      break;
    case 'about':
      alert('Taxa - AI-Powered Notebook\n版本 1.0.0');
      break;
    case 'undo':
    case 'redo':
    case 'cut':
    case 'copy':
    case 'paste':
      document.execCommand(action);
      break;
  }
}
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

.menu-left {
  display: flex;
  align-items: center;
  gap: 2px;
  -webkit-app-region: no-drag;
}

.app-title {
  font-weight: 600;
  margin-right: 16px;
  font-size: 13px;
  color: var(--text-primary);
}

.menu-wrapper {
  position: relative;
}

.menu-item {
  padding: 4px 10px;
  font-size: 13px;
  color: var(--text-secondary);
  border-radius: 4px;
  cursor: pointer;
}

.menu-item:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 200px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  z-index: 100;
  margin-top: 2px;
}

.menu-separator {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.dropdown-item {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-primary);
  background: none;
  border: none;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  align-items: center;
  text-align: left;
}

.dropdown-item:hover {
  background: var(--accent-color);
  color: white;
}

.has-submenu {
  position: relative;
}

.submenu-arrow {
  font-size: 10px;
  margin-left: auto;
  padding-left: 16px;
}

.submenu {
  position: absolute;
  left: 100%;
  top: -4px;
  min-width: 180px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
}

.shortcut {
  font-size: 11px;
  opacity: 0.7;
  margin-left: 20px;
}
</style>
