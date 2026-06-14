// src/stores/settings.ts
import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { LlmProvider, LlmProviderForm } from '@/types/settings';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const providers = ref<LlmProvider[]>([]);
  const theme = ref<'light' | 'dark' | 'system'>('system');

  async function loadProviders() {
    providers.value = await invoke<LlmProvider[]>('list_providers');
  }

  async function saveProvider(form: LlmProviderForm & { id?: string }) {
    const id = form.id || form.name.toLowerCase().replace(/[\s+]+/g, '-') + '-' + Date.now();
    const apiUrl = form.api_url || getDefaultUrl(form.provider_type);
    const modelName = form.model_name || getDefaultModel(form.provider_type);

    // When editing and api_key is empty, keep the existing key
    const apiKey = form.api_key || (form.id ? '' : '');

    await invoke('save_provider', {
      config: {
        id,
        name: form.name,
        provider_type: form.provider_type,
        api_url: apiUrl,
        api_key: apiKey,
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
      apiUrl: form.api_url || getDefaultUrl(form.provider_type),
      apiKey: form.api_key,
      modelName: form.model_name || getDefaultModel(form.provider_type),
    });
  }

  function getDefaultUrl(type: string): string {
    switch (type) {
      case 'claude': return 'https://api.anthropic.com';
      case 'openai': return 'https://api.openai.com';
      case 'glm': return 'https://open.bigmodel.cn/api/paas/v4';
      case 'deepseek': return 'https://api.deepseek.com';
      case 'minimax': return 'https://api.minimax.chat/v1';
      case 'kimi': return 'https://api.moonshot.cn/v1';
      default: return 'https://api.example.com/v1';
    }
  }

  function getDefaultModel(type: string): string {
    switch (type) {
      case 'claude': return 'claude-sonnet-4-6';
      case 'openai': return 'gpt-4o';
      case 'glm': return 'glm-4';
      case 'deepseek': return 'deepseek-chat';
      case 'minimax': return 'MiniMax-Text-01';
      case 'kimi': return 'moonshot-v1-8k';
      default: return 'model-name';
    }
  }

  return { providers, theme, loadProviders, saveProvider, deleteProvider, reorderProviders, testProvider };
});
