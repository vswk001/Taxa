// Editor store + TabBar regressions: pinned tabs must survive bulk closes,
// the active tab falls back to the last survivor, and bulk closes flush the
// pending autosave BEFORE removing tabs (the old code nulled currentNote
// without flushing and lost the last second of edits).
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia, type Pinia } from 'pinia';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }));

import { useEditorStore } from '@/stores/editor';
import { useNotebookStore } from '@/stores/notebook';
import TabBar from '@/components/editor/TabBar.vue';
import i18n from '@/i18n';

const tt = (key: string) => i18n.global.t(key);

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('editor store closeTabs', () => {
  it('keeps pinned tabs on close-all', () => {
    const store = useEditorStore();
    store.openTab('a', 'A');
    store.openTab('b', 'B');
    store.openTab('c', 'C');
    store.togglePin('b');

    store.closeTabs(['a', 'b', 'c']);
    expect(store.openTabs.map((t) => t.id)).toEqual(['b']);
  });

  it('falls back the active tab to the last survivor', () => {
    const store = useEditorStore();
    store.openTab('a', 'A');
    store.openTab('b', 'B');
    store.openTab('c', 'C');
    store.setActiveTab('b');

    store.closeTabs(['b']);
    expect(store.activeTabId).toBe('c');
  });

  it('clears the active tab when everything closes', () => {
    const store = useEditorStore();
    store.openTab('a', 'A');
    store.closeTabs(['a']);
    expect(store.activeTabId).toBeNull();
  });
});

function mountBar(pinia: Pinia): VueWrapper {
  return mount(TabBar, { global: { plugins: [pinia, i18n] } });
}

describe('TabBar context menu actions', () => {
  function setup() {
    const pinia = createPinia();
    setActivePinia(pinia);
    const editor = useEditorStore();
    editor.openTab('a', 'A');
    editor.openTab('b', 'B');
    editor.openTab('c', 'C');
    editor.setActiveTab('b');
    editor.togglePin('a');
    const notebook = useNotebookStore();
    const flush = vi.fn().mockResolvedValue(undefined);
    notebook.flushPendingSave = flush;
    const openNote = vi.fn().mockResolvedValue(undefined);
    notebook.openNote = openNote;
    const wrapper = mountBar(pinia);
    return { wrapper, editor, flush, openNote };
  }

  async function openMenuOn(wrapper: VueWrapper, title: string) {
    const tabs = wrapper.findAll('.tab');
    const tab = tabs.find((w) => w.text().toLowerCase().includes(title.toLowerCase()))!;
    await tab.trigger('contextmenu', { clientX: 10, clientY: 10 });
    await flushPromises();
  }

  it('close-right removes tabs to the right but keeps pinned ones and flushes first', async () => {
    const { wrapper, editor, flush } = setup();
    await openMenuOn(wrapper, 'B');
    await wrapper
      .findAll('.tab-menu button')
      .find((b) => b.text() === tt('editor.closeRight'))!
      .trigger('click');

    expect(flush).toHaveBeenCalled();
    expect(editor.openTabs.map((t) => t.id).sort()).toEqual(['a', 'b']);
  });

  it('close-others keeps only the target and pinned tabs', async () => {
    const { wrapper, editor } = setup();
    await openMenuOn(wrapper, 'c');
    await wrapper
      .findAll('.tab-menu button')
      .find((b) => b.text() === tt('editor.closeOthers'))!
      .trigger('click');
    expect(editor.openTabs.map((t) => t.id).sort()).toEqual(['a', 'c']);
  });

  it('close-all keeps pinned tabs', async () => {
    const { wrapper, editor } = setup();
    await openMenuOn(wrapper, 'b');
    await wrapper
      .findAll('.tab-menu button')
      .find((b) => b.text() === tt('editor.closeAll'))!
      .trigger('click');
    expect(editor.openTabs.map((t) => t.id)).toEqual(['a']);
  });

  it('pointerdown inside the menu does not close it (menu items must stay clickable)', async () => {
    const { wrapper } = setup();
    await openMenuOn(wrapper, 'b');
    wrapper.get('.tab-menu').element.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    await flushPromises();
    expect(wrapper.find('.tab-menu').exists()).toBe(true);
    wrapper.unmount();
  });
});
