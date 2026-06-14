<template>
  <div class="operation-card">
    <div class="card-title">{{ t('ai.aiSuggestion') }}</div>
    <div v-if="isOptimize" class="card-body">
      <div class="field">
        <span class="label">{{ t('ai.titleLabel') }}</span>
        <input v-model="local.title" class="field-input" />
      </div>
      <div class="field">
        <span class="label">{{ t('ai.changesLabel') }}</span>
        <span class="changes-text">{{ local.content?.slice(0, 200) }}{{ (local.content?.length || 0) > 200 ? '...' : '' }}</span>
      </div>
    </div>
    <div v-else class="card-body">
      <div class="field">
        <span class="label">{{ t('ai.actionLabel') }}</span>
        <select v-model="local.action" class="field-select">
          <option value="create">{{ t('ai.actionCreate') }}</option>
          <option value="append">{{ t('ai.actionAppend') }}</option>
        </select>
      </div>
      <div class="field">
        <span class="label">{{ t('ai.titleLabel') }}</span>
        <input v-model="local.title" class="field-input" />
      </div>
      <div class="field">
        <span class="label">{{ t('ai.folderLabel') }}</span>
        <input v-model="local.folder" class="field-input" />
      </div>
      <div class="field">
        <span class="label">{{ t('ai.tagsLabel') }}</span>
        <input v-model="tagsText" class="field-input" :placeholder="t('ai.tagsLabel')" />
      </div>
    </div>
    <div class="card-actions">
      <button class="btn-confirm" @click="handleConfirm">{{ isOptimize ? t('ai.applyOptimize') : t('ai.confirmAction') }}</button>
      <button class="btn-dismiss" @click="emit('dismiss')">{{ t('ai.dismissAction') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AiSuggestion } from '@/types/ai';

const { t } = useI18n();
const isOptimize = computed(() => props.suggestion.action === 'optimize');

const props = defineProps<{ suggestion: AiSuggestion }>();
const emit = defineEmits<{ confirm: [suggestion: AiSuggestion]; dismiss: [] }>();

const local = reactive({
  action: props.suggestion.action,
  title: props.suggestion.title || '',
  folder: props.suggestion.folder || '',
  tags: [...(props.suggestion.tags || [])],
  content: props.suggestion.content || '',
  target_note_id: props.suggestion.target_note_id,
  confidence: props.suggestion.confidence,
});

const tagsText = ref(local.tags.join(', '));

watch(tagsText, (val) => {
  local.tags = val.split(',').map(t => t.trim()).filter(Boolean);
});

function handleConfirm() {
  emit('confirm', { ...local });
}
</script>

<style scoped>
.operation-card {
  margin-top: 8px; border: 1px solid var(--border-color); border-radius: 8px;
  overflow: hidden; font-size: 12px;
}
.card-title { background: var(--accent-color); color: white; padding: 6px 10px; font-weight: 600; }
.card-body { padding: 8px 10px; }
.field { padding: 3px 0; display: flex; align-items: center; gap: 6px; }
.label { color: var(--text-secondary); white-space: nowrap; min-width: 36px; }
.field-input {
  flex: 1; padding: 3px 6px; font-size: 12px;
  border: 1px solid var(--border-color); border-radius: 4px;
  background: var(--bg-primary); color: var(--text-primary);
  outline: none; min-width: 0;
}
.field-input:focus { border-color: var(--accent-color); }
.field-select {
  padding: 3px 6px; font-size: 12px;
  border: 1px solid var(--border-color); border-radius: 4px;
  background: var(--bg-primary); color: var(--text-primary);
  outline: none; cursor: pointer;
}
.card-actions { display: flex; gap: 8px; padding: 8px 10px; border-top: 1px solid var(--border-color); }
.changes-text { flex: 1; font-size: 12px; color: var(--text-secondary); white-space: pre-wrap; max-height: 80px; overflow-y: auto; }
.btn-confirm { flex: 1; padding: 6px; background: var(--success-color); color: white; border-radius: 4px; font-size: 12px; }
.btn-dismiss { flex: 1; padding: 6px; background: var(--bg-secondary); color: var(--text-secondary); border-radius: 4px; font-size: 12px; }
</style>
