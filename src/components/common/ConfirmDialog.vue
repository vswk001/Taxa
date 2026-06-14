<template>
  <div v-if="visible" class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-box">
      <div class="dialog-message">
        <span class="dialog-icon">{{ kind === 'danger' ? '⚠️' : '❓' }}</span>
        {{ message }}
      </div>
      <div class="dialog-actions">
        <button class="dialog-btn cancel" @click="emit('cancel')">{{ t('common.cancel') }}</button>
        <button class="dialog-btn confirm" :class="kind" @click="emit('confirm')">{{ t('common.confirm') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

defineProps<{
  visible: boolean;
  message: string;
  kind?: 'danger' | 'warning';
}>();

const emit = defineEmits<{ confirm: []; cancel: [] }>();
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}
.dialog-box {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 24px;
  min-width: 320px;
  max-width: 420px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  text-align: center;
}
.dialog-icon {
  font-size: 20px;
  flex-shrink: 0;
}
.dialog-message {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.6;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.dialog-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
}
.dialog-btn {
  padding: 8px 24px;
  font-size: 13px;
  border-radius: 6px;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: opacity 0.15s;
}
.dialog-btn:hover { opacity: 0.85; }
.dialog-btn.cancel {
  background: var(--bg-secondary);
  color: var(--text-primary);
}
.dialog-btn.confirm {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}
.dialog-btn.confirm.danger {
  background: var(--danger-color);
  border-color: var(--danger-color);
}
</style>
