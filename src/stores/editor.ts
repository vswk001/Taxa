// src/stores/editor.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface EditorTab {
  id: string;
  title: string;
  /** Pinned tabs keep their place and are exempt from bulk closes. */
  pinned: boolean;
}

export const useEditorStore = defineStore('editor', () => {
  const openTabs = ref<EditorTab[]>([]);
  const activeTabId = ref<string | null>(null);
  const isModified = ref(false);
  let savePromise: Promise<void> | null = null;

  function openTab(id: string, title: string) {
    const existing = openTabs.value.find((t) => t.id === id);
    if (existing) {
      // Keep the pinned flag; refresh the title (renames come through here).
      existing.title = title;
    } else {
      openTabs.value.push({ id, title, pinned: false });
    }
    activeTabId.value = id;
  }

  function closeTab(id: string) {
    closeTabs([id]);
  }

  /** Bulk close. Pinned tabs are kept unless explicitly included. The active
   *  tab falls back to the last remaining tab (or null when all close). */
  function closeTabs(ids: string[], includePinned = false) {
    const closeSet = new Set(ids);
    const removed = new Set(
      openTabs.value.filter((t) => closeSet.has(t.id) && (includePinned || !t.pinned)).map((t) => t.id),
    );
    if (!removed.size) return;
    openTabs.value = openTabs.value.filter((t) => !removed.has(t.id));
    if (activeTabId.value && removed.has(activeTabId.value)) {
      activeTabId.value = openTabs.value.length > 0
        ? openTabs.value[openTabs.value.length - 1].id
        : null;
    }
  }

  function togglePin(id: string) {
    const tab = openTabs.value.find((t) => t.id === id);
    if (tab) tab.pinned = !tab.pinned;
  }

  function setActiveTab(id: string) {
    if (openTabs.value.find((t) => t.id === id)) {
      activeTabId.value = id;
    }
  }

  function setSavePromise(p: Promise<void> | null) {
    savePromise = p;
  }

  async function waitForSave() {
    if (savePromise) {
      await savePromise;
      savePromise = null;
    }
  }

  function updateTabTitle(id: string, title: string) {
    const tab = openTabs.value.find((t) => t.id === id);
    if (tab) tab.title = title;
  }

  return {
    openTabs, activeTabId, isModified,
    openTab, closeTab, closeTabs, togglePin, setActiveTab,
    setSavePromise, waitForSave, updateTabTitle,
  };
});
