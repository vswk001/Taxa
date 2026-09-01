// The v0.4.0 regression: the capture-phase document pointerdown listener
// removed the context menu from the DOM before a menu item's click could
// dispatch — every right-click action silently did nothing.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a), convertFileSrc: (p: string) => p }));
let notesChangedHandler: (() => void) | null = null;
vi.mock('@tauri-apps/api/event', () => ({
  listen: async (_name: string, cb: () => void) => {
    if (_name === 'notes-changed') notesChangedHandler = cb;
    return () => {};
  },
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ message: vi.fn() }));

import NoteTree from '@/components/tree/NoteTree.vue';
import TreeNode from '@/components/tree/TreeNode.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import { useNotebookStore } from '@/stores/notebook';
import i18n from '@/i18n';
import type { Note } from '@/types/notebook';

function mountTree(): VueWrapper {
  const pinia = createPinia();
  setActivePinia(pinia);
  // One folder so a (stubbed) TreeNode renders and can emit the context event.
  useNotebookStore().folders = [
    { name: 'F', path: 'f', children: [], note_count: 0 },
  ];
  return mount(NoteTree, {
    global: { plugins: [pinia, i18n] },
    shallow: true, // stub TreeNode/dialogs; the menu itself is in this template
  });
}

function noteFixture(): Note {
  return {
    id: 'n1', path: 'A.md', title: 'A', folder: '', tags: [],
    created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z',
    word_count: 1, summary: null, ai_categorized: false,
  };
}

/** Right-click a note via the (stubbed) TreeNode's event. */
async function openNoteMenu(wrapper: VueWrapper, note: Note) {
  const node = wrapper.findComponent(TreeNode);
  node.vm.$emit('contextmenu-note', { preventDefault() {}, stopPropagation() {} }, note);
  await flushPromises();
}

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((cmd: string) => {
    if (cmd === 'get_folder_tree') return [];
    if (cmd === 'list_notes') return [];
    return undefined;
  });
  notesChangedHandler = null;
  setActivePinia(createPinia());
});

describe('note tree context menu', () => {
  it('shows the menu on right-click and closes it on an outside pointerdown', async () => {
    const wrapper = mountTree();
    await openNoteMenu(wrapper, noteFixture());
    expect(wrapper.find('.context-menu').exists()).toBe(true);

    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }));
    await flushPromises();
    expect(wrapper.find('.context-menu').exists()).toBe(false);
  });

  it('keeps the menu open for pointerdown INSIDE it so the item click fires (v0.4.0 regression)', async () => {
    const wrapper = mountTree();
    await openNoteMenu(wrapper, noteFixture());

    const menu = wrapper.find('.context-menu');
    // pointerdown on a menu item (capture listener must pass it through)…
    menu.get('button.danger').element.dispatchEvent(
      new Event('pointerdown', { bubbles: true }),
    );
    await flushPromises();
    expect(wrapper.find('.context-menu').exists()).toBe(true);

    // …then the click reaches the handler: menu closes, confirm dialog opens.
    await menu.get('button.danger').trigger('click');
    expect(wrapper.find('.context-menu').exists()).toBe(false);
    const confirm = wrapper.findComponent(ConfirmDialog);
    expect(confirm.props('visible')).toBe(true);
  });

  it('Escape closes the menu', async () => {
    const wrapper = mountTree();
    await openNoteMenu(wrapper, noteFixture());
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    await flushPromises();
    expect(wrapper.find('.context-menu').exists()).toBe(false);
  });
});

describe('tree refresh on cross-window note changes', () => {
  it('reloads folders and notes when the notes-changed event fires (quick capture)', async () => {
    const wrapper = mountTree();
    await flushPromises();
    expect(notesChangedHandler).toBeTruthy();

    const notebook = useNotebookStore();
    const calls: string[] = [];
    notebook.loadFolderTree = async () => { calls.push('tree'); };
    notebook.loadAllNotes = async () => { calls.push('notes'); };

    notesChangedHandler!();
    await flushPromises();
    expect(calls).toEqual(['tree', 'notes']);
    wrapper.unmount();
  });
});
