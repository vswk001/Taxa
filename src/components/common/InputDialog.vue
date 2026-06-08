<template>
  <div v-if="visible" class="dialog-overlay" @click.self="$emit('cancel')">
    <div class="dialog-box">
      <div class="dialog-title">{{ title }}</div>
      <input
        ref="inputRef"
        v-model="value"
        class="dialog-input"
        :placeholder="placeholder"
        @keydown.enter="$emit('confirm', value)"
        @keydown.escape="$emit('cancel')"
      />
      <div class="dialog-actions">
        <button class="dialog-btn cancel" @click="$emit('cancel')">{{ t('common.cancel') }}</button>
        <button class="dialog-btn confirm" @click="$emit('confirm', value)">{{ t('common.confirm') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();;

const props = defineProps<{
  visible: boolean;
  title: string;
  placeholder?: string;
  defaultValue?: string;
}>();

defineEmits<{ confirm: [value: string]; cancel: [] }>();

const value = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(() => props.visible, (v) => {
  if (v) {
    value.value = props.defaultValue || '';
    nextTick(() => inputRef.value?.focus());
  }
});
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
  padding: 20px 24px;
  min-width: 340px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.dialog-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 14px;
}

.dialog-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  outline: none;
  box-sizing: border-box;
}

.dialog-input:focus {
  border-color: var(--accent-color);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.dialog-btn {
  padding: 6px 16px;
  font-size: 13px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  cursor: pointer;
}

.dialog-btn.cancel {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.dialog-btn.confirm {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.dialog-btn:hover {
  opacity: 0.85;
}
</style>
