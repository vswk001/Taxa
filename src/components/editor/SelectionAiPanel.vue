<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="panelRef"
      class="selection-ai"
      :style="{ left: x + 'px', top: y + 'px' }"
      @mousedown.prevent
    >
      <template v-if="state === 'idle'">
        <button v-for="a in actions" :key="a" @click="emit('action', a)">{{ t('editor.aiAction.' + a) }}</button>
      </template>
      <template v-else-if="state === 'loading'">
        <span class="loading">{{ t('editor.aiAction.working') }}</span>
      </template>
      <template v-else-if="state === 'done'">
        <pre class="result">{{ result }}</pre>
        <div class="result-actions">
          <button class="primary" @click="emit('apply', 'replace')">{{ t('editor.aiAction.replace') }}</button>
          <button @click="emit('apply', 'insert')">{{ t('editor.aiAction.insertAfter') }}</button>
          <button @click="emit('cancel')">{{ t('common.cancel') }}</button>
        </div>
      </template>
      <template v-else>
        <span class="error">{{ error }}</span>
        <button @click="emit('cancel')">{{ t('common.cancel') }}</button>
      </template>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  visible: boolean;
  x: number;
  y: number;
  state: 'idle' | 'loading' | 'done' | 'error';
  result: string;
  error: string;
}>();

const emit = defineEmits<{
  action: [action: string];
  apply: [mode: 'replace' | 'insert'];
  cancel: [];
}>();

const { t } = useI18n();
const panelRef = ref<HTMLElement | null>(null);
const actions = ['polish', 'translate', 'explain', 'expand'] as const;

// Keep the panel on screen when the selection is near an edge.
watch(
  () => [props.visible, props.x, props.y] as const,
  async () => {
    if (!props.visible) return;
    await nextTick();
    const el = panelRef.value;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    if (rect.bottom > window.innerHeight - 8) {
      el.style.top = `${Math.max(8, props.y - rect.height - 24)}px`;
    }
    if (rect.right > window.innerWidth - 8) {
      el.style.left = `${Math.max(8, window.innerWidth - rect.width - 8)}px`;
    }
  },
);
</script>

<style scoped>
.selection-ai {
  position: fixed;
  z-index: 300;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
  max-width: 420px;
}

.selection-ai button {
  padding: 4px 10px;
  font-size: 12px;
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 5px;
  cursor: pointer;
  color: var(--text-primary);
  white-space: nowrap;
}

.selection-ai button:hover {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.selection-ai button.primary {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.loading,
.error {
  font-size: 12px;
  padding: 2px 6px;
}

.error {
  color: var(--danger-color);
}

.result {
  font-family: var(--font-sans);
  font-size: 12px;
  line-height: 1.5;
  max-height: 140px;
  overflow-y: auto;
  margin: 0;
  padding: 6px;
  white-space: pre-wrap;
  background: var(--bg-secondary);
  border-radius: 5px;
  color: var(--text-primary);
}

.result-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}
</style>
