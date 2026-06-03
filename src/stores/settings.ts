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

  async function saveProvider(form: LlmProviderForm) {
    const id = form.name.toLowerCase().replace(/\s+/g, '-') + '-' + Date.now();
    // Rust save_provider expects param named "config"
    await invoke('save_provider', {
      config: {
        id,
        name: form.name,
        provider_type: form.provider_type,
        api_url: form.api_url || getDefaultUrl(form.provider_type),
        api_key: form.api_key,
        model_name: form.model_name || getDefaultModel(form.provider_type),
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
      default: return 'https://api.example.com';
    }
  }

  function getDefaultModel(type: string): string {
    switch (type) {
      case 'claude': return 'claude-sonnet-4-6';
      case 'openai': return 'gpt-4o';
      default: return '';
    }
  }

  return { providers, theme, loadProviders, saveProvider, deleteProvider, testProvider };
});
