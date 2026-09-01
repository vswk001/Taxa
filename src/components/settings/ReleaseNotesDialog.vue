<template>
  <Teleport to="body">
    <div v-if="visible" class="notes-overlay" @click.self="emit('close')">
      <div class="notes-dialog">
        <div class="notes-header">
          <span>{{ t('settings.releaseNotes') }} — v{{ version }}</span>
          <button class="close-btn" @click="emit('close')">×</button>
        </div>
        <div class="notes-body">
          <div v-if="markdown" class="notes-md" v-html="rendered"></div>
          <div v-else class="notes-empty">{{ t('settings.releaseNotesEmpty') }}</div>
        </div>
        <div class="notes-footer">
          <button class="primary" @click="emit('install')">{{ t('settings.updateInstall') }}</button>
          <button @click="emit('close')">{{ t('common.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{ visible: boolean; version: string; markdown: string }>();
const emit = defineEmits<{ close: []; install: [] }>();
const { t } = useI18n();

/** Tiny markdown subset renderer for release notes. Input is HTML-escaped
 *  first, so the generated markup is safe to v-html. Links degrade to their
 *  label text (no in-app navigation for external URLs). */
const rendered = computed(() => {
  const esc = props.markdown
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
  return esc
    .split(/\r?\n/)
    .map((line) => {
      let l = line;
      l = l.replace(/\[([^\]]*)\]\([^)]*\)/g, '$1');
      l = l.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
      l = l.replace(/`([^`]+)`/g, '<code>$1</code>');
      const heading = /^(#{1,4})\s+(.*)$/.exec(l);
      if (heading) {
        const level = Math.min(heading[1].length + 1, 5);
        return `<div class="md-h${level}">${heading[2]}</div>`;
      }
      if (/^\s*[-*]\s+/.test(l)) {
        return `<div class="md-li">• ${l.replace(/^\s*[-*]\s+/, '')}</div>`;
      }
      if (/^\s*\|/.test(l)) {
        return `<div class="md-row">${l}</div>`;
      }
      if (/^---+$/.test(l.trim())) {
        return '<hr class="md-hr" />';
      }
      if (!l.trim()) return '';
      return `<div class="md-p">${l}</div>`;
    })
    .join('\n');
});
</script>

<style scoped>
.notes-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 220;
  display: flex;
  align-items: center;
  justify-content: center;
}

.notes-dialog {
  width: 640px;
  max-height: 70vh;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.notes-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-secondary);
}

.notes-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  font-size: 13px;
  line-height: 1.65;
  color: var(--text-primary);
}

.notes-empty {
  text-align: center;
  color: var(--text-secondary);
  padding: 32px 0;
}

.notes-md :deep(.md-h2),
.notes-md :deep(.md-h3) {
  font-weight: 600;
  margin: 14px 0 6px;
}

.notes-md :deep(.md-h2) {
  font-size: 15px;
}

.notes-md :deep(.md-h3) {
  font-size: 14px;
}

.notes-md :deep(.md-h4),
.notes-md :deep(.md-h5) {
  font-weight: 600;
  font-size: 13px;
  margin: 12px 0 4px;
}

.notes-md :deep(.md-li) {
  padding-left: 14px;
  margin: 3px 0;
}

.notes-md :deep(.md-p) {
  margin: 6px 0;
}

.notes-md :deep(.md-row) {
  font-family: var(--font-mono);
  font-size: 12px;
  white-space: pre-wrap;
  color: var(--text-secondary);
}

.notes-md :deep(.md-hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 12px 0;
}

.notes-md :deep(code) {
  background: var(--bg-secondary);
  border-radius: 4px;
  padding: 1px 5px;
  font-family: var(--font-mono);
  font-size: 12px;
}

.notes-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 18px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.notes-footer button {
  padding: 7px 18px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: none;
  cursor: pointer;
  color: var(--text-primary);
}

.notes-footer button.primary {
  background: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}
</style>
