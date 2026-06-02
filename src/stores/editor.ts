// src/stores/editor.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useEditorStore = defineStore('editor', () => {
  const openTabs = ref<{ id: string; title: string }[]>([]);
  const activeTabId = ref<string | null>(null);
  const isModified = ref(false);

  function openTab(id: string, title: string) {
    if (!openTabs.value.find(t => t.id === id)) {
      openTabs.value.push({ id, title });
    }
    activeTabId.value = id;
  }

  function closeTab(id: string) {
    openTabs.value = openTabs.value.filter(t => t.id !== id);
    if (activeTabId.value === id) {
      activeTabId.value = openTabs.value.length > 0 ? openTabs.value[openTabs.value.length - 1].id : null;
    }
  }

  function setActiveTab(id: string) {
    if (openTabs.value.find(t => t.id === id)) {
      activeTabId.value = id;
    }
  }

  return { openTabs, activeTabId, isModified, openTab, closeTab, setActiveTab };
});
