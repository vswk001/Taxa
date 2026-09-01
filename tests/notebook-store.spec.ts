// Regression tests for the notebook store — each case locks a real bug that
// was fixed: sending '' content wiped non-open notes (P0), updateNoteContent
// silently no-oped, openNote dropped pending edits, stale searches won.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { useNotebookStore } from '@/stores/notebook';
import type { Note, NoteWithContent } from '@/types/notebook';

function note(id: string, title: string, folder = ''): Note {
  return {
    id, path: `${folder}/${title}.md`, title, folder, tags: [],
    created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z',
    word_count: 1, summary: null, ai_categorized: false,
  };
}

function withContent(n: Note, content: string): NoteWithContent {
  return { note: n, content };
}

beforeEach(() => {
  invoke.mockReset();
  setActivePinia(createPinia());
});

describe('renameNote / updateNoteTags content handling', () => {
  it('OMITS the content field for a note that is not the open note (sending "" wiped files)', async () => {
    const store = useNotebookStore();
    store.notes = [note('n1', 'A')];
    store.currentNote = withContent(note('n2', 'B'), 'open note body');

    invoke.mockResolvedValue(note('n1', 'Renamed'));
    await store.renameNote('n1', 'Renamed');

    const req = invoke.mock.calls.find((c) => c[0] === 'update_note')![1].req;
    expect(req.title).toBe('Renamed');
    expect('content' in req).toBe(false);
  });

  it('includes the live content when renaming the open note', async () => {
    const store = useNotebookStore();
    const open = note('n2', 'B');
    store.notes = [open];
    store.currentNote = withContent(open, 'open note body');

    invoke.mockResolvedValue(note('n2', 'Renamed'));
    await store.renameNote('n2', 'Renamed');

    const req = invoke.mock.calls.find((c) => c[0] === 'update_note')![1].req;
    expect(req.content).toBe('open note body');
  });

  it('updateNoteTags also omits content for non-open notes', async () => {
    const store = useNotebookStore();
    store.notes = [note('n1', 'A')];

    invoke.mockResolvedValue(note('n1', 'A'));
    await store.updateNoteTags('n1', ['x']);

    const req = invoke.mock.calls.find((c) => c[0] === 'update_note')![1].req;
    expect(req.tags).toEqual(['x']);
    expect('content' in req).toBe(false);
  });
});

describe('updateNoteContent', () => {
  it('updates by id even when no note is open (the old currentNote guard silently no-oped AI applies)', async () => {
    const store = useNotebookStore();
    store.currentNote = null;

    invoke.mockResolvedValue(note('n1', 'A'));
    await store.updateNoteContent('n1', 'new body');

    expect(invoke).toHaveBeenCalledWith('update_note', { req: { id: 'n1', content: 'new body' } });
  });
});

describe('openNote', () => {
  it('flushes the pending autosave BEFORE fetching the next note', async () => {
    const store = useNotebookStore();
    const order: string[] = [];
    store.registerPendingSave(async () => {
      order.push('flush');
    });
    invoke.mockImplementation(async (cmd: string) => {
      order.push(cmd);
      return withContent(note('n9', 'Next'), 'next body');
    });

    await store.openNote('n9');
    expect(order[0]).toBe('flush');
    expect(order[1]).toBe('get_note');
    expect(store.currentNote?.note.id).toBe('n9');
  });
});

describe('search sequencing', () => {
  it('discards a stale response that resolves after a newer search', async () => {
    const store = useNotebookStore();
    let resolveSlow!: (v: unknown) => void;
    invoke.mockImplementationOnce(() => new Promise((r) => { resolveSlow = r; }));
    invoke.mockImplementationOnce(async () => [{ id: 'fresh' }]);

    const slow = store.search('first');
    const fresh = store.search('second');
    resolveSlow([{ id: 'stale' }]);
    await Promise.all([slow, fresh]);

    expect(store.searchResults).toEqual([{ id: 'fresh' }]);
  });
});
