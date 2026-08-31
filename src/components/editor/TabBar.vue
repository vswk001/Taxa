<template>
  <div class="tab-bar">
    <div
      v-for="tab in editorStore.openTabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === editorStore.activeTabId, pinned: tab.pinned }"
      @click="handleTabClick(tab.id)"
      @auxclick.middle.prevent="handleCloseTab(tab.id)"
      @contextmenu.prevent="openMenu($event, tab.id)"
    >
      <span v-if="tab.pinned" class="pin-icon" :title="t('editor.unpinTab')">📌</span>
      <span
        v-if="editorStore.isModified && tab.id === editorStore.activeTabId"
        class="modified-dot"
        :title="t('editor.unsavedChanges')"
      ></span>
      <span class="tab-title">{{ tab.title }}</span>
      <button
        v-if="!tab.pinned"
        class="tab-close"
        @click.stop="handleCloseTab(tab.id)"
        :title="t('editor.closeTab')"
      >×</button>
    </div>
    <button class="tab-new" @click="handleNewNote" :title="t('editor.newNoteTitle')">+</button>

    <div
      v-if="menu.show"
      ref="menuRef"
      class="tab-menu"
      :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      @click.stop
    >
      <button v-if="menu.targetId !== '__graph__'" @click="runAction('toggle-pin')">
        {{ isTargetPinned ? t('editor.unpinTab') : t('editor.pinTab') }}
      </button>
      <button @click="runAction('close')">{{ t('editor.closeTab') }}</button>
      <button @click="runAction('close-others')">{{ t('editor.closeOthers') }}</button>
      <button @click="runAction('close-right')">{{ t('editor.closeRight') }}</button>
      <div class="menu-separator"></div>
      <button @click="runAction('close-all')">{{ t('editor.closeAll') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useEditorStore } from '@/stores/editor';
import { useNotebookStore } from '@/stores/notebook';

const { t } = useI18n();
const editorStore = useEditorStore();
const notebookStore = useNotebookStore();

const menu = ref({ show: false, x: 0, y: 0, targetId: '' });
const menuRef = ref<HTMLElement | null>(null);

const isTargetPinned = computed(
  () => editorStore.openTabs.find((tb) => tb.id === menu.value.targetId)?.pinned ?? false,
);

function openMenu(e: MouseEvent, tabId: string) {
  menu.value = { show: true, x: e.clientX, y: e.clientY, targetId: tabId };
}

/** Close on any click outside the menu itself or on Escape. Clicks inside
 *  pass through so the action's click handler can run (the menu is closed
 *  by the action, not by the pointerdown). */
function onDocPointerDown(e: PointerEvent) {
  if (!menu.value.show) return;
  if (menuRef.value?.contains(e.target as Node)) return;
  menu.value.show = false;
}

function onDocKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape' && menu.value.show) menu.value.show = false;
}

onMounted(() => {
  document.addEventListener('pointerdown', onDocPointerDown, true);
  document.addEventListener('keydown', onDocKeyDown, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onDocPointerDown, true);
  document.removeEventListener('keydown', onDocKeyDown, true);
});

async function runAction(action: string) {
  const targetId = menu.value.targetId;
  menu.value.show = false;
  if (!targetId) return;

  const tabs = editorStore.openTabs;
  const idx = tabs.findIndex((tb) => tb.id === targetId);
  if (idx < 0) return;

  switch (action) {
    case 'toggle-pin':
      editorStore.togglePin(targetId);
      return;
    case 'close':
      await handleCloseTab(targetId);
      return;
    case 'close-others':
      await closeTabs(tabs.filter((tb) => tb.id !== targetId).map((tb) => tb.id));
      return;
    case 'close-right':
      await closeTabs(tabs.slice(idx + 1).map((tb) => tb.id));
      return;
    case 'close-all':
      await closeTabs(tabs.map((tb) => tb.id));
      return;
  }
}

/** Close a batch of tabs: flush pending edits first, then re-target the
 *  visible note exactly once. Pinned tabs survive (closeTabs honors them). */
async function closeTabs(ids: string[]) {
  await notebookStore.flushPendingSave();
  const wasActive = ids.includes(editorStore.activeTabId ?? '');
  editorStore.closeTabs(ids);
  if (wasActive) await activateCurrent();
}

async function activateCurrent() {
  const active = editorStore.activeTabId;
  if (active && active !== '__graph__') {
    await notebookStore.openNote(active);
  } else if (!active) {
    notebookStore.currentNote = null;
  }
}

function handleTabClick(tabId: string) {
  if (editorStore.activeTabId === tabId) return;
  editorStore.setActiveTab(tabId);
}

async function handleNewNote() {
  try {
    const folder = notebookStore.currentFolder || (notebookStore.folders[0]?.path || t('tree.Uncategorized'));
    await notebookStore.createNote(folder, t('editor.newTab'), '');
  } catch (error) {
    console.error('Failed to create new note:', error);
  }
}

async function handleCloseTab(tabId: string) {
  // Pinned tabs keep their × hidden, but be safe if this ever fires for one.
  const tab = editorStore.openTabs.find((tb) => tb.id === tabId);
  if (tab?.pinned) return;
  await closeTabs([tabId]);
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  height: 40px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
  overflow-y: hidden;
  position: relative;
}

.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: 100%;
  font-size: 13px;
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  color: var(--text-secondary);
  white-space: nowrap;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.tab:hover {
  background: var(--bg-primary);
}

.tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-color);
}

.tab.pinned {
  max-width: 180px;
}

.modified-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent-color);
  flex-shrink: 0;
}

.pin-icon {
  font-size: 11px;
  line-height: 1;
}

.tab-title {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  font-size: 18px;
  color: var(--text-secondary);
  padding: 0 2px;
  background: none;
  border: none;
  cursor: pointer;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.tab-close:hover {
  color: var(--danger-color);
  background: var(--border-color);
}

.tab-new {
  padding: 0 16px;
  font-size: 20px;
  color: var(--text-secondary);
  height: 100%;
  background: none;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-new:hover {
  color: var(--text-primary);
  background: var(--border-color);
}

.tab-menu {
  position: fixed;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 150px;
  padding: 4px;
  display: flex;
  flex-direction: column;
}

.menu-separator {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.tab-menu button {
  padding: 7px 12px;
  text-align: left;
  background: none;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
}

.tab-menu button:hover {
  background: var(--bg-secondary);
}
</style>
