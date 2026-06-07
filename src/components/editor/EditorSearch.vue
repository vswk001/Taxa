<template>
  <div v-if="visible" class="editor-search" @keydown="handleKeydown">
    <input
      ref="inputRef"
      v-model="query"
      class="search-input"
      placeholder="搜索..."
      @input="debouncedSearch"
    />
    <span class="search-count">{{ matchText }}</span>
    <button class="search-btn" @click="findPrev" title="上一个 (Shift+Enter)">▲</button>
    <button class="search-btn" @click="findNext" title="下一个 (Enter)">▼</button>
    <button class="search-btn close-btn" @click="emit('close')" title="关闭 (Escape)">✕</button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue';

const props = defineProps<{ visible: boolean; container: HTMLElement | null }>();
const emit = defineEmits<{ close: [] }>();

const query = ref('');
const matchRanges = ref<Range[]>([]);
const matchIndex = ref(-1);
const inputRef = ref<HTMLInputElement | null>(null);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const supportsHighlight = typeof CSS !== 'undefined' && 'highlights' in CSS;

const matchText = computed(() => {
  if (!query.value) return '';
  if (matchRanges.value.length === 0) return '无匹配';
  return `${matchIndex.value + 1}/${matchRanges.value.length}`;
});

watch(() => props.visible, (v) => {
  if (v) {
    nextTick(() => inputRef.value?.focus());
  } else {
    clearHighlight();
  }
});

onBeforeUnmount(() => {
  clearHighlight();
  if (debounceTimer) clearTimeout(debounceTimer);
});

function debouncedSearch() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(performSearch, 150);
}

function performSearch() {
  clearHighlight();
  matchRanges.value = [];
  matchIndex.value = -1;
  if (!query.value || !props.container) return;

  const ranges = findMatches(props.container, query.value);
  matchRanges.value = ranges;

  if (ranges.length > 0) {
    matchIndex.value = 0;
    applyHighlights();
  }
}

function findMatches(root: HTMLElement, q: string): Range[] {
  const ranges: Range[] = [];
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const nodes: { node: Text; start: number }[] = [];
  let totalLen = 0;

  while (walker.nextNode()) {
    const textNode = walker.currentNode as Text;
    nodes.push({ node: textNode, start: totalLen });
    totalLen += (textNode.textContent || '').length;
  }

  const fullText = nodes.map(n => n.node.textContent || '').join('');
  const lowerText = fullText.toLowerCase();
  const lowerQ = q.toLowerCase();
  let offset = 0;

  while ((offset = lowerText.indexOf(lowerQ, offset)) !== -1) {
    const matchStart = offset;
    const matchEnd = offset + q.length;
    offset = matchEnd;

    try {
      const range = document.createRange();
      let started = false;
      let done = false;

      for (const entry of nodes) {
        const nodeText = entry.node.textContent || '';
        const nodeStart = entry.start;
        const nodeEnd = nodeStart + nodeText.length;

        if (!started && nodeEnd > matchStart) {
          const localStart = matchStart - nodeStart;
          if (nodeEnd >= matchEnd) {
            range.setStart(entry.node, localStart);
            range.setEnd(entry.node, matchEnd - nodeStart);
            started = true;
            done = true;
            break;
          } else {
            range.setStart(entry.node, localStart);
            started = true;
          }
        }

        if (started && nodeEnd >= matchEnd) {
          range.setEnd(entry.node, matchEnd - nodeStart);
          done = true;
          break;
        }
      }

      if (done) ranges.push(range);
    } catch { /* skip invalid ranges */ }
  }

  return ranges;
}

function applyHighlights() {
  if (matchRanges.value.length === 0) return;

  if (supportsHighlight) {
    const allHighlight = new Highlight(...matchRanges.value);
    CSS.highlights.set('search-matches', allHighlight);
    updateCurrentHighlight();
  } else {
    scrollToCurrent();
  }
}

function updateCurrentHighlight() {
  if (!supportsHighlight) return;
  CSS.highlights.delete('search-current');
  if (matchIndex.value >= 0 && matchIndex.value < matchRanges.value.length) {
    const currentHighlight = new Highlight(matchRanges.value[matchIndex.value]);
    CSS.highlights.set('search-current', currentHighlight);
  }
  scrollToCurrent();
}

function scrollToCurrent() {
  if (matchIndex.value < 0 || matchIndex.value >= matchRanges.value.length) return;
  const range = matchRanges.value[matchIndex.value];
  const el = range.commonAncestorContainer.parentElement;
  if (el) el.scrollIntoView({ block: 'center', behavior: 'smooth' });
}

function findNext() {
  if (matchRanges.value.length === 0) return;
  matchIndex.value = (matchIndex.value + 1) % matchRanges.value.length;
  updateCurrentHighlight();
}

function findPrev() {
  if (matchRanges.value.length === 0) return;
  matchIndex.value = (matchIndex.value - 1 + matchRanges.value.length) % matchRanges.value.length;
  updateCurrentHighlight();
}

function clearHighlight() {
  if (supportsHighlight) {
    CSS.highlights.delete('search-matches');
    CSS.highlights.delete('search-current');
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    emit('close');
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (e.shiftKey) findPrev();
    else findNext();
  } else if (e.key === 'F3') {
    e.preventDefault();
    if (e.shiftKey) findPrev();
    else findNext();
  }
}
</script>

<style scoped>
.editor-search {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.search-input {
  width: 200px;
  padding: 4px 8px;
  font-size: 13px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  outline: none;
}

.search-input:focus {
  border-color: var(--accent-color);
}

.search-count {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 50px;
  text-align: center;
}

.search-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  border-radius: 4px;
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  cursor: pointer;
}

.search-btn:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

.close-btn:hover {
  color: var(--danger-color);
}
</style>

<style>
::highlight(search-matches) {
  background-color: rgba(255, 200, 0, 0.45);
  color: inherit;
}
::highlight(search-current) {
  background-color: #f08000;
  color: #fff;
}
</style>
