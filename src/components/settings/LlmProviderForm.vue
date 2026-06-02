<!-- src/components/settings/LlmProviderForm.vue -->
<template>
  <div class="provider-form">
    <div class="form-group">
      <label>名称</label>
      <input v-model="form.name" placeholder="例如: 我的 Claude" />
    </div>
    <div class="form-group">
      <label>类型</label>
      <select v-model="form.provider_type">
        <option value="claude">Claude (Anthropic)</option>
        <option value="openai">OpenAI</option>
        <option value="openai_compatible">OpenAI 兼容 (GLM/MiniMax/Kimi 等)</option>
        <option value="custom">自定义</option>
      </select>
    </div>
    <div class="form-group">
      <label>API URL</label>
      <input v-model="form.api_url" :placeholder="defaultUrl" />
      <span class="hint">留空使用默认: {{ defaultUrl }}</span>
    </div>
    <div class="form-group">
      <label>API Key</label>
      <input v-model="form.api_key" type="password" placeholder="sk-..." />
    </div>
    <div class="form-group">
      <label>模型</label>
      <input v-model="form.model_name" :placeholder="defaultModel" />
    </div>
    <div class="form-group">
      <label><input type="checkbox" v-model="form.is_default" /> 设为默认</label>
    </div>
    <div class="form-actions">
      <button class="btn-test" @click="testConnection" :disabled="testing">{{ testing ? '测试中...' : '测试连接' }}</button>
      <button class="btn-save" @click="emit('save', { ...form })">保存</button>
      <button class="btn-cancel" @click="emit('cancel')">取消</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { LlmProviderForm } from '@/types/settings';
import { useSettingsStore } from '@/stores/settings';

const emit = defineEmits<{ save: [form: LlmProviderForm]; cancel: [] }>();
const settingsStore = useSettingsStore();
const testing = ref(false);

const form = ref<LlmProviderForm>({
  name: '', provider_type: 'openai_compatible',
  api_url: '', api_key: '', model_name: '',
  is_default: false,
});

const defaultUrl = computed(() => {
  switch (form.value.provider_type) {
    case 'claude': return 'https://api.anthropic.com';
    case 'openai': return 'https://api.openai.com';
    default: return 'https://api.example.com';
  }
});

const defaultModel = computed(() => {
  switch (form.value.provider_type) {
    case 'claude': return 'claude-sonnet-4-6';
    case 'openai': return 'gpt-4o';
    default: return '';
  }
});

async function testConnection() {
  testing.value = true;
  try {
    const ok = await settingsStore.testProvider(form.value);
    alert(ok ? '连接成功!' : '连接失败');
  } catch (e: any) {
    alert('连接失败: ' + e);
  } finally {
    testing.value = false;
  }
}
</script>

<style scoped>
.provider-form { padding: 16px; }
.form-group { margin-bottom: 14px; }
.form-group label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 4px; }
.form-group input, .form-group select {
  width: 100%; padding: 8px; border: 1px solid var(--border-color);
  border-radius: var(--radius); font-size: 13px; background: var(--bg-primary);
  color: var(--text-primary);
}
.hint { font-size: 11px; color: var(--text-secondary); }
.form-actions { display: flex; gap: 8px; margin-top: 16px; }
.btn-test { padding: 8px 16px; background: var(--bg-secondary); border-radius: var(--radius); font-size: 13px; }
.btn-save { padding: 8px 16px; background: var(--accent-color); color: white; border-radius: var(--radius); font-size: 13px; }
.btn-cancel { padding: 8px 16px; font-size: 13px; }
</style>
