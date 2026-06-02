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
    await invoke('save_provider', {
      config: { id, ...form, enabled: true },
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
      apiUrl: form.api_url,
      apiKey: form.api_key,
      modelName: form.model_name,
    });
  }

  return { providers, theme, loadProviders, saveProvider, deleteProvider, testProvider };
});
