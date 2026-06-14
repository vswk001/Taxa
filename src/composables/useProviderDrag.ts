import { ref, type Ref } from 'vue';
import type { LlmProvider } from '@/types/settings';

/**
 * Drag-and-drop reordering for the LLM provider list.
 * `providers` is a ref to the (Pinia) store list — updates are optimistic,
 * then `persist` saves the new order to the backend.
 */
export function useProviderDrag(
  providers: Ref<LlmProvider[]>,
  persist: (orderedIds: string[]) => Promise<void>,
) {
  const dragIndex = ref<number | null>(null);
  const overIndex = ref<number | null>(null);
  let saving = false;

  function onDragStart(index: number) {
    dragIndex.value = index;
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (dragIndex.value !== null && dragIndex.value !== index) {
      overIndex.value = index;
    }
  }

  async function onDrop(index: number) {
    const from = dragIndex.value;
    reset();
    if (from === null || from === index || saving) return;

    saving = true;
    const list = [...providers.value];
    const [moved] = list.splice(from, 1);
    list.splice(index, 0, moved);
    // Optimistic update so the UI snaps immediately.
    providers.value = list;
    try {
      await persist(list.map(p => p.id));
    } finally {
      saving = false;
    }
  }

  function reset() {
    dragIndex.value = null;
    overIndex.value = null;
  }

  return { dragIndex, overIndex, onDragStart, onDragOver, onDrop, onDragEnd: reset };
}
