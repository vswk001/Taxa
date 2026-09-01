// AI store regressions: the shared streamUnlisten let a finished request
// unhook a newer one, cancel didn't reach the backend, and Reset events
// (failed attempt) left stale reasoning in the message.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';

const invoke = vi.fn();
let listenHandler: ((e: { payload: { seq: number; event: never } }) => void) | null = null;
const unlisten = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock('@tauri-apps/api/event', () => ({
  listen: async (_name: string, cb: (e: never) => void) => {
    listenHandler = cb as typeof listenHandler;
    return unlisten;
  },
}));

import { useAiStore } from '@/stores/ai';
import type { OrganizeResult } from '@/types/ai-extended';

const okResult: OrganizeResult = {
  action: 'create', title: 'T', folder: '', tags: [], content: 'c',
  target_note_id: null, complexity: 'simple',
};

function emit(seq: number, event: unknown) {
  listenHandler?.({ payload: { seq, event: event as never } });
}

beforeEach(() => {
  invoke.mockReset();
  unlisten.mockReset();
  listenHandler = null;
  setActivePinia(createPinia());
  vi.useFakeTimers();
});

describe('cancel', () => {
  it('aborts the backend request (ai_cancel) and marks the message errored', async () => {
    const store = useAiStore();
    invoke.mockImplementation((cmd: string) =>
      cmd === 'ai_process_input'
        ? new Promise(() => { /* hangs like a long stream */ })
        : Promise.resolve(undefined),
    );
    const pending = store.submitInput('hello');
    await vi.advanceTimersByTimeAsync(10); // let the listener attach

    store.cancel();
    expect(invoke).toHaveBeenCalledWith('ai_cancel', { seq: 1 });

    const last = store.messages[store.messages.length - 1];
    expect(last.status).toBe('error');
    await Promise.race([pending, vi.advanceTimersByTimeAsync(150_000)]);
  });
});

describe('stream events', () => {
  it('Reset clears the partial reasoning of a failed provider attempt', async () => {
    const store = useAiStore();
    let resolveInvoke!: (v: OrganizeResult) => void;
    invoke.mockImplementation((cmd: string) =>
      cmd === 'ai_process_input' ? new Promise((r) => { resolveInvoke = r; }) : undefined,
    );
    const pending = store.submitInput('hello');
    await vi.advanceTimersByTimeAsync(10);

    emit(1, { type: 'Reasoning', text: 'partial' });
    emit(1, { type: 'Reset', text: null });
    let msg = store.messages.find((m) => m.role === 'assistant')!;
    expect(msg.reasoning).toBeUndefined();

    emit(1, { type: 'Fallback', text: { failed: 'A', next: 'B' } });
    msg = store.messages.find((m) => m.role === 'assistant')!;
    expect(msg.fallbackInfo).toEqual({ failed: 'A', next: 'B' });

    resolveInvoke(okResult);
    await pending;
  });
});

describe('listener lifetime', () => {
  it('unlistens its own request-local listener after completion', async () => {
    const store = useAiStore();
    invoke.mockResolvedValue(okResult);
    const pending = store.submitInput('hello');
    await vi.advanceTimersByTimeAsync(10);
    expect(listenHandler).toBeTruthy();

    await pending;
    await vi.advanceTimersByTimeAsync(0);
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it('ignores stream events for a different seq', async () => {
    const store = useAiStore();
    let resolveInvoke!: (v: OrganizeResult) => void;
    invoke.mockImplementation((cmd: string) =>
      cmd === 'ai_process_input' ? new Promise((r) => { resolveInvoke = r; }) : undefined,
    );
    const pending = store.submitInput('hello');
    await vi.advanceTimersByTimeAsync(10);

    emit(99, { type: 'Reasoning', text: 'other request' });
    const msg = store.messages.find((m) => m.role === 'assistant')!;
    expect(msg.reasoning).toBeUndefined();

    resolveInvoke(okResult);
    await pending;
  });
});
