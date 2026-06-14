<template>
  <div class="provider-form">
    <div class="form-group">
      <label>{{ t('llmForm.name') }}</label>
      <input v-model="form.name" :placeholder="t('llmForm.namePlaceholder')" />
    </div>
    <div class="form-group">
      <label>{{ t('llmForm.type') }}</label>
      <select v-model="form.provider_type" @change="updateDefaults">
        <option value="claude">{{ t('llmForm.typeClaude') }}</option>
        <option value="openai">{{ t('llmForm.typeOpenai') }}</option>
        <option value="openai_compatible">{{ t('llmForm.typeOpenaiCompatible') }}</option>
        <option value="glm">{{ t('llmForm.typeGlm') }}</option>
        <option value="deepseek">{{ t('llmForm.typeDeepseek') }}</option>
        <option value="minimax">{{ t('llmForm.typeMinimax') }}</option>
        <option value="kimi">{{ t('llmForm.typeKimi') }}</option>
        <option value="custom">{{ t('llmForm.typeCustom') }}</option>
      </select>
    </div>
    <div class="form-group">
      <label>{{ t('llmForm.apiUrl') }}</label>
      <input v-model="form.api_url" :placeholder="urlPlaceholder" />
      <span class="hint">{{ urlHint }}</span>
    </div>
    <div class="form-group">
      <label>{{ t('llmForm.apiKey') }}</label>
      <input v-model="form.api_key" type="password" :placeholder="initialData?.id ? t('llmForm.apiKeyPlaceholder') : 'sk-...'" />
    </div>
    <div class="form-group">
      <label>{{ t('llmForm.model') }}</label>
      <input v-model="form.model_name" :placeholder="modelPlaceholder" />
    </div>
    <div class="checkbox-group">
      <input id="is_default" type="checkbox" v-model="form.is_default" />
      <label for="is_default">{{ t('llmForm.setDefault') }}</label>
    </div>
    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
    <div v-if="successMsg" class="success-msg">{{ successMsg }}</div>
    <div class="form-actions">
      <button class="btn-test" @click="testConnection" :disabled="testing">
        {{ testing ? t('llmForm.testing') : t('llmForm.testConnection') }}
      </button>
      <button class="btn-save" @click="handleSave">{{ t('llmForm.save') }}</button>
      <button class="btn-cancel" @click="emit('cancel')">{{ t('common.cancel') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { LlmProviderForm } from '@/types/settings';
import { useSettingsStore } from '@/stores/settings';

const { t } = useI18n();

const props = defineProps<{ initialData?: LlmProviderForm & { id?: string } }>();
const emit = defineEmits<{ save: [form: LlmProviderForm & { id?: string }]; cancel: [] }>();
const settingsStore = useSettingsStore();
const testing = ref(false);
const errorMsg = ref('');
const successMsg = ref('');

const form = ref<LlmProviderForm>({
  name: props.initialData?.name || '',
  provider_type: props.initialData?.provider_type || 'openai_compatible',
  api_url: props.initialData?.api_url || '',
  api_key: props.initialData?.api_key || '',
  model_name: props.initialData?.model_name || '',
  is_default: props.initialData?.is_default ?? false,
});

const urlPlaceholder = computed(() => {
  switch (form.value.provider_type) {
    case 'claude': return 'https://api.anthropic.com';
    case 'openai': return 'https://api.openai.com';
    case 'glm': return 'https://open.bigmodel.cn/api/paas/v4';
    case 'deepseek': return 'https://api.deepseek.com';
    case 'minimax': return 'https://api.minimax.chat/v1';
    case 'kimi': return 'https://api.moonshot.cn/v1';
    default: return 'https://api.example.com/v1';
  }
});

const urlHint = computed(() => {
  switch (form.value.provider_type) {
    case 'claude': return t('llmForm.urlHintClaude');
    case 'openai': return t('llmForm.urlHintOpenai');
    case 'glm': return t('llmForm.urlHintGlm');
    case 'deepseek': return t('llmForm.urlHintDeepseek');
    case 'openai_compatible': return t('llmForm.urlHintCompatible');
    default: return t('llmForm.urlHintCustom');
  }
});

const modelPlaceholder = computed(() => {
  switch (form.value.provider_type) {
    case 'claude': return 'claude-sonnet-4-6';
    case 'openai': return 'gpt-4o';
    case 'glm': return 'glm-4';
    case 'deepseek': return 'deepseek-chat';
    case 'minimax': return 'MiniMax-Text-01';
    case 'kimi': return 'moonshot-v1-8k';
    default: return 'model-name';
  }
});

function updateDefaults() {
  if (!form.value.api_url) {
    form.value.api_url = '';
  }
  if (!form.value.model_name) {
    form.value.model_name = '';
  }
}

async function testConnection() {
  testing.value = true;
  errorMsg.value = '';
  successMsg.value = '';
  try {
    const ok = await settingsStore.testProvider(form.value);
    if (ok) {
      successMsg.value = t('llmForm.connectionSuccess');
    } else {
      errorMsg.value = t('llmForm.connectionFailed');
    }
  } catch (e: any) {
    errorMsg.value = t('llmForm.connectionFailedPrefix') + (e.message || String(e));
  } finally {
    testing.value = false;
  }
}

function handleSave() {
  if (!form.value.name.trim()) { errorMsg.value = t('llmForm.nameRequired'); return; }
  if (!form.value.api_key.trim() && !props.initialData?.id) { errorMsg.value = t('llmForm.apiKeyRequired'); return; }
  if (!form.value.model_name.trim()) {
    form.value.model_name = modelPlaceholder.value;
  }
  if (!form.value.api_url.trim()) {
    form.value.api_url = urlPlaceholder.value;
  }
  errorMsg.value = '';
  emit('save', { ...form.value, id: props.initialData?.id });
}
</script>

<style scoped>
.provider-form { padding: 16px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 4px; color: var(--text-primary); }
.form-group input, .form-group select {
  width: 100%; padding: 8px; border: 1px solid var(--border-color);
  border-radius: var(--radius); font-size: 13px; background: var(--bg-primary); color: var(--text-primary);
  box-sizing: border-box;
}
.form-group input:focus, .form-group select:focus { outline: none; border-color: var(--accent-color); }
.checkbox-group {
  display: flex; align-items: center; gap: 8px; margin-bottom: 14px;
}
.checkbox-group input {
  width: auto; margin: 0; cursor: pointer; accent-color: var(--accent-color);
}
.checkbox-group label {
  margin: 0; font-weight: 500; font-size: 13px; color: var(--text-primary); cursor: pointer;
}
.hint { font-size: 11px; color: var(--text-secondary); display: block; margin-top: 2px; }
.form-actions { display: flex; gap: 8px; margin-top: 16px; }
.btn-test { padding: 8px 16px; background: var(--bg-secondary); border-radius: var(--radius); font-size: 13px; border: 1px solid var(--border-color); cursor: pointer; color: var(--text-primary); }
.btn-test:disabled { opacity: 0.5; }
.btn-save { padding: 8px 16px; background: var(--accent-color); color: white; border-radius: var(--radius); font-size: 13px; border: none; cursor: pointer; }
.btn-cancel { padding: 8px 16px; font-size: 13px; background: none; border: 1px solid var(--border-color); border-radius: var(--radius); cursor: pointer; color: var(--text-primary); }
.error-msg { color: var(--danger-color, red); font-size: 12px; margin-bottom: 8px; padding: 6px 10px; background: rgba(255,0,0,0.05); border-radius: 4px; }
.success-msg { color: #2e7d32; font-size: 12px; margin-bottom: 8px; padding: 6px 10px; background: rgba(46,125,50,0.08); border-radius: 4px; }
</style>
