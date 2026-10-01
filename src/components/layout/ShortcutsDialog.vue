<template>
  <Teleport to="body">
    <div v-if="visible" class="sc-overlay" @click.self="emit('close')">
      <div class="sc-dialog">
        <div class="sc-header">
          <span>{{ t('shortcuts.title') }}</span>
          <button class="close-btn" @click="emit('close')">×</button>
        </div>
        <div class="sc-body">
          <div v-for="group in groups" :key="group.label" class="sc-group">
            <div class="sc-group-title">{{ t(group.label) }}</div>
            <div v-for="item in group.items" :key="item.key" class="sc-row">
              <span class="sc-desc">{{ t(item.desc) }}</span>
              <kbd class="sc-key">{{ item.key }}</kbd>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();

const groups = [
  {
    label: 'shortcuts.global',
    items: [
      { key: 'Ctrl+P', desc: 'shortcuts.palette' },
      { key: 'Ctrl+K', desc: 'shortcuts.search' },
      { key: 'Ctrl+G', desc: 'shortcuts.graph' },
      { key: 'Ctrl+B', desc: 'shortcuts.sidebar' },
      { key: 'Ctrl+Alt+N', desc: 'shortcuts.quickCapture' },
      { key: '?', desc: 'shortcuts.help' },
    ],
  },
  {
    label: 'shortcuts.editorGroup',
    items: [
      { key: 'Ctrl+S', desc: 'shortcuts.save' },
      { key: 'Ctrl+F', desc: 'shortcuts.find' },
      { key: '[[', desc: 'shortcuts.link' },
    ],
  },
  {
    label: 'shortcuts.tabsGroup',
    items: [
      { key: t('shortcuts.rightClick'), desc: 'shortcuts.tabMenu' },
      { key: t('shortcuts.middleClick'), desc: 'shortcuts.closeTabKey' },
    ],
  },
];
</script>

<style scoped>
.sc-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 230;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sc-dialog {
  width: 440px;
  max-height: 480px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.sc-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 13px 18px;
  font-weight: 600;
  font-size: 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-secondary);
}

.sc-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 18px;
}

.sc-group-title {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 12px 0 6px;
}

.sc-group:first-child .sc-group-title {
  margin-top: 0;
}

.sc-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 0;
  font-size: 13px;
  color: var(--text-primary);
}

.sc-key {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  padding: 2px 8px;
  color: var(--text-secondary);
}
</style>
