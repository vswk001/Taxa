<!-- src/components/ai/ChatInput.vue -->
<template>
  <div class="chat-input-wrap">
    <div v-if="attachments.length" class="attachment-list">
      <div v-for="(f, i) in attachments" :key="i" class="attachment-chip">
        <span class="chip-icon">📎</span>
        <span class="chip-name">{{ f.name }}</span>
        <button class="chip-remove" @click="attachments.splice(i, 1)">×</button>
      </div>
    </div>
    <div class="chat-input">
      <textarea
        v-model="text"
        :placeholder="t('ai.inputPlaceholder')"
        :disabled="disabled"
        @keydown.enter.exact.prevent="submit"
        @drop.prevent="handleDrop"
        @paste="handlePaste"
      />
      <div class="input-actions">
        <button class="attach-btn" :disabled="disabled" @click="pickFile" :title="t('ai.uploadFile')">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48"/></svg>
        </button>
        <button class="send-btn" :disabled="disabled || (!text.trim() && !attachments.length)" @click="submit">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import type { FileAttachment } from '@/types/ai';

const { t } = useI18n();
const MAX_FILE_CHARS = 50000;

defineProps<{ disabled: boolean }>();
const emit = defineEmits<{ submit: [content: string, attachments: FileAttachment[]] }>();
const text = ref('');
const attachments = ref<FileAttachment[]>([]);

function sanitizeContent(raw: string): string {
  // Remove null bytes and limit size
  let s = raw.replace(/\0/g, '');
  if (s.length > MAX_FILE_CHARS) {
    s = s.slice(0, MAX_FILE_CHARS) + '\n' + t('ai.fileTruncated');
  }
  return s;
}

async function pickFile() {
  const selected = await open({
    multiple: true,
    filters: [{ name: t('ai.fileFilter'), extensions: ['md', 'txt', 'json', 'csv', 'xml', 'html', 'css', 'js', 'ts', 'py', 'rs', 'java', 'go', 'log', 'yaml', 'yml', 'toml', 'ini', 'conf', 'sh', 'bat'] }],
  });
  if (!selected) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  for (const p of paths) {
    try {
      const pathStr = typeof p === 'string' ? p : String(p);
      const name = pathStr.split(/[/\\]/).pop() || 'file';
      const bytes = await readFile(pathStr);
      const content = sanitizeContent(new TextDecoder('utf-8', { fatal: false }).decode(bytes));
      attachments.value.push({ name, content });
    } catch (e) {
      console.error('Failed to read file:', e);
    }
  }
}

async function handleDrop(e: DragEvent) {
  const files = e.dataTransfer?.files;
  if (!files?.length) return;
  for (const f of Array.from(files)) {
    try {
      const content = sanitizeContent(await f.text());
      attachments.value.push({ name: f.name, content });
    } catch (e) {
      console.error('Failed to read dropped file:', e);
    }
  }
}

function handlePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items;
  if (!items) return;
  for (const item of Array.from(items)) {
    if (item.kind === 'file') {
      const file = item.getAsFile();
      if (file) {
        file.text().then((raw: string) => {
          attachments.value.push({ name: file.name, content: sanitizeContent(raw) });
        });
      }
    }
  }
}

function submit() {
  if (!text.value.trim() && !attachments.value.length) return;
  emit('submit', text.value.trim(), [...attachments.value]);
  text.value = '';
  attachments.value = [];
}
</script>

<style scoped>
.chat-input-wrap {
  border-top: 1px solid var(--border-color);
}
.attachment-list {
  display: flex; flex-wrap: wrap; gap: 6px;
  padding: 8px 12px 0;
}
.attachment-chip {
  display: flex; align-items: center; gap: 4px;
  padding: 3px 8px; font-size: 12px;
  background: var(--bg-secondary); border: 1px solid var(--border-color);
  border-radius: 12px; color: var(--text-primary);
  max-width: 180px;
}
.chip-icon { font-size: 11px; flex-shrink: 0; }
.chip-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.chip-remove {
  font-size: 14px; color: var(--text-secondary); background: none;
  border: none; cursor: pointer; padding: 0 2px; line-height: 1;
}
.chip-remove:hover { color: var(--danger-color); }
.chat-input {
  display: flex; gap: 8px; padding: 12px; align-items: flex-end;
}
.chat-input textarea {
  flex: 1; resize: none; border: 1px solid var(--border-color);
  border-radius: var(--radius); padding: 8px; font-size: 13px;
  background: var(--bg-primary); color: var(--text-primary);
  min-height: 40px; max-height: 120px;
}
.chat-input textarea:focus { outline: none; border-color: var(--accent-color); }
.input-actions { display: flex; flex-direction: column; gap: 4px; }
.attach-btn {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 28px; border: none; background: none;
  color: var(--text-secondary); cursor: pointer; border-radius: 4px;
}
.attach-btn:hover:not(:disabled) { background: var(--bg-secondary); color: var(--text-primary); }
.attach-btn:disabled { opacity: 0.4; }
.send-btn {
  display: flex; align-items: center; justify-content: center;
  width: 32px; height: 28px; background: var(--accent-color); color: white;
  border: none; border-radius: var(--radius); cursor: pointer;
}
.send-btn:hover:not(:disabled) { opacity: 0.9; }
.send-btn:disabled { opacity: 0.5; }
</style>
