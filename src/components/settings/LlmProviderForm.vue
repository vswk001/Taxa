<template>
  <div class="provider-form">
    <div class="form-group">
      <label>名称</label>
      <input v-model="form.name" placeholder="例如: 我的 Claude" />
    </div>
    <div class="form-group">
      <label>类型</label>
      <select v-model="form.provider_type" @change="updateDefaults">
        <option value="claude">Claude (Anthropic)</option>
        <option value="openai">OpenAI</option>
        <option value="openai_compatible">OpenAI 兼容</option>
        <option value="glm">智谱 GLM</option>
        <option value="deepseek">DeepSeek</option>
        <option value="minimax">MiniMax</option>
        <option value="kimi">Kimi (Moonshot)</option>
        <option value="custom">自定义</option>
      </select>
    </div>
    <div class="form-group">
      <label>API 地址</label>
      <input v-model="form.api_url" :placeholder="urlPlaceholder" />
      <span class="hint">{{ urlHint }}</span>
    </div>
    <div class="form-group">
      <label>API Key</label>
      <input v-model="form.api_key" type="password" :placeholder="initialData?.id ? '留空保持不变' : 'sk-...'" />
    </div>
    <div class="form-group">
      <label>模型</label>
      <input v-model="form.model_name" :placeholder="modelPlaceholder" />
    </div>
    <div class="form-group">
      <label><input type="checkbox" v-model="form.is_default" /> 设为默认</label>
    </div>
    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
    <div class="form-actions">
      <button class="btn-test" @click="testConnection" :disabled="testing">
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button class="btn-save" @click="handleSave">保存</button>
      <button class="btn-cancel" @click="emit('cancel')">取消</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { LlmProviderForm } from '@/types/settings';
import { useSettingsStore } from '@/stores/settings';

const props = defineProps<{ initialData?: LlmProviderForm & { id?: string } }>();
const emit = defineEmits<{ save: [form: LlmProviderForm & { id?: string }]; cancel: [] }>();
const settingsStore = useSettingsStore();
const testing = ref(false);
const errorMsg = ref('');

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
    case 'claude': return 'Anthropic API 地址，留空使用默认';
    case 'openai': return 'OpenAI API 地址，留空使用默认';
    case 'glm': return '智谱 API 地址，留空使用默认';
    case 'deepseek': return 'DeepSeek API 地址，留空使用默认';
    case 'openai_compatible': return '填写兼容 OpenAI 格式的完整 API 地址';
    default: return '填写完整的 API 地址';
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
  try {
    const ok = await settingsStore.testProvider(form.value);
    if (ok) {
      errorMsg.value = '';
      // Show success briefly
      testing.value = false;
    } else {
      errorMsg.value = '连接失败：未收到有效响应';
    }
  } catch (e: any) {
    errorMsg.value = '连接失败: ' + (e.message || String(e));
  } finally {
    testing.value = false;
  }
}

function handleSave() {
  if (!form.value.name.trim()) { errorMsg.value = '请输入名称'; return; }
  if (!form.value.api_key.trim() && !props.initialData?.id) { errorMsg.value = '请输入 API Key'; return; }
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
.hint { font-size: 11px; color: var(--text-secondary); display: block; margin-top: 2px; }
.form-actions { display: flex; gap: 8px; margin-top: 16px; }
.btn-test { padding: 8px 16px; background: var(--bg-secondary); border-radius: var(--radius); font-size: 13px; border: 1px solid var(--border-color); cursor: pointer; color: var(--text-primary); }
.btn-test:disabled { opacity: 0.5; }
.btn-save { padding: 8px 16px; background: var(--accent-color); color: white; border-radius: var(--radius); font-size: 13px; border: none; cursor: pointer; }
.btn-cancel { padding: 8px 16px; font-size: 13px; background: none; border: 1px solid var(--border-color); border-radius: var(--radius); cursor: pointer; color: var(--text-primary); }
.error-msg { color: var(--danger-color, red); font-size: 12px; margin-bottom: 8px; padding: 6px 10px; background: rgba(255,0,0,0.05); border-radius: 4px; }
</style>
