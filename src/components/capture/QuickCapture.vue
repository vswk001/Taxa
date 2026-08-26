<template>
  <div class="quick-capture" @keydown="onKeyDown">
    <textarea
      ref="inputRef"
      v-model="text"
      class="capture-input"
      :placeholder="t('quickCapture.placeholder')"
      :disabled="busy"
      rows="6"
    />
    <div class="capture-footer">
      <span class="hint">{{ t('quickCapture.hint') }}</span>
      <span v-if="status === 'error'" class="status error">{{ errorText }}</span>
      <span v-else-if="status === 'done'" class="status done">{{ doneText }}</span>
      <button class="submit" :disabled="busy || !text.trim()" @click="submit">
        {{ busy ? t('quickCapture.working') : t('quickCapture.submit') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const { t, locale } = useI18n();
const text = ref('');
const busy = ref(false);
const status = ref<'idle' | 'done' | 'error'>('idle');
const errorText = ref('');
const organized = ref(false);
const inputRef = ref<HTMLTextAreaElement | null>(null);

const doneText = computed(() =>
  organized.value ? t('quickCapture.organized') : t('quickCapture.savedToInbox'),
);

function hide() {
  void getCurrentWindow().hide();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    hide();
  }
}

async function submit() {
  const content = text.value.trim();
  if (!content || busy.value) return;
  busy.value = true;
  status.value = 'idle';
  try {
    const note = await invoke<{ title: string; folder: string }>('quick_capture', {
      content,
      locale: locale.value,
    });
    organized.value = note.folder !== 'Inbox';
    status.value = 'done';
    text.value = '';
    // Brief feedback, then hide until the next hotkey press.
    setTimeout(() => {
      status.value = 'idle';
      hide();
    }, 1200);
  } catch (e) {
    status.value = 'error';
    errorText.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<style scoped>
.quick-capture {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  overflow: hidden;
}

.capture-input {
  flex: 1;
  border: none;
  outline: none;
  resize: none;
  padding: 14px 16px;
  font-size: 14px;
  line-height: 1.6;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.capture-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.hint {
  flex: 1;
  font-size: 11px;
  color: var(--text-secondary);
}

.status {
  font-size: 12px;
}

.status.done {
  color: var(--accent-color);
}

.status.error {
  color: var(--danger-color);
}

.submit {
  padding: 6px 16px;
  font-size: 13px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.submit:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
