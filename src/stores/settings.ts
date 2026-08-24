// src/stores/settings.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { LlmProvider, LlmProviderForm, ProviderType } from '@/types/settings';
import { invoke } from '@tauri-apps/api/core';

/** Default API endpoints per provider type (shared with LlmProviderForm). */
export const DEFAULT_PROVIDER_URLS: Record<ProviderType, string> = {
  claude: 'https://api.anthropic.com',
  openai: 'https://api.openai.com',
  glm: 'https://open.bigmodel.cn/api/paas/v4',
  deepseek: 'https://api.deepseek.com',
  minimax: 'https://api.minimax.chat/v1',
  kimi: 'https://api.moonshot.cn/v1',
  openai_compatible: 'https://api.example.com/v1',
  custom: 'https://api.example.com/v1',
};

/** Default model names per provider type (shared with LlmProviderForm). */
export const DEFAULT_PROVIDER_MODELS: Record<ProviderType, string> = {
  claude: 'claude-sonnet-4-6',
  openai: 'gpt-4o',
  glm: 'glm-4',
  deepseek: 'deepseek-chat',
  minimax: 'MiniMax-Text-01',
  kimi: 'moonshot-v1-8k',
  openai_compatible: 'model-name',
  custom: 'model-name',
};

export const useSettingsStore = defineStore('settings', () => {
  const providers = ref<LlmProvider[]>([]);
  const theme = ref<'light' | 'dark' | 'system'>('system');

  async function loadProviders() {
    providers.value = await invoke<LlmProvider[]>('list_providers');
  }

  async function saveProvider(form: LlmProviderForm & { id?: string }) {
    const id = form.id || form.name.toLowerCase().replace(/[\s+]+/g, '-') + '-' + Date.now();
    const apiUrl = form.api_url || DEFAULT_PROVIDER_URLS[form.provider_type];
    const modelName = form.model_name || DEFAULT_PROVIDER_MODELS[form.provider_type];

    // An empty api_key means "keep the stored one" — the backend preserves it.
    await invoke('save_provider', {
      config: {
        id,
        name: form.name,
        provider_type: form.provider_type,
        api_url: apiUrl,
        api_key: form.api_key,
        model_name: modelName,
        is_default: form.is_default,
        enabled: true,
      },
    });
    await loadProviders();
  }

  async function deleteProvider(id: string) {
    await invoke('delete_provider', { id });
    await loadProviders();
  }

  async function reorderProviders(orderedIds: string[]) {
    await invoke('reorder_providers', { orderedIds });
    await loadProviders();
  }

  async function testProvider(form: LlmProviderForm) {
    return invoke<boolean>('ai_test_provider', {
      providerType: form.provider_type,
      apiUrl: form.api_url || DEFAULT_PROVIDER_URLS[form.provider_type],
      apiKey: form.api_key,
      modelName: form.model_name || DEFAULT_PROVIDER_MODELS[form.provider_type],
    });
  }

  async function setDefault(id: string) {
    const provider = providers.value.find(p => p.id === id);
    if (!provider) return;
    await invoke('save_provider', {
      config: {
        id: provider.id,
        name: provider.name,
        provider_type: provider.provider_type,
        api_url: provider.api_url,
        api_key: '', // empty = keep stored key
        model_name: provider.model_name,
        is_default: true,
        enabled: provider.enabled,
      },
    });
    await loadProviders();
  }

  return {
    providers, theme,
    loadProviders, saveProvider, deleteProvider, reorderProviders, testProvider, setDefault,
  };
});
