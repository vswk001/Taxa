<template>
  <div class="ai-sidebar">
    <div class="sidebar-header">
      <span>{{ t('ai.assistant') }}</span>
      <div class="header-actions">
        <button v-if="aiStore.messages.length > 0" @click="aiStore.clearMessages()" :title="t('ai.clearChat')">🗑</button>
        <button :class="{ active: showConfig }" @click="showConfig = !showConfig" :title="t('ai.llmConfig')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
      </div>
    </div>

    <!-- LLM Config Panel -->
    <div v-if="showConfig" class="config-panel">
      <div v-if="settingsStore.providers.length === 0" class="no-providers">
        <p>{{ t('ai.noProviders') }}</p>
        <p class="hint">{{ t('ai.noProvidersHint') }}</p>
      </div>
      <div class="provider-list">
        <div v-for="p in settingsStore.providers" :key="p.id" class="provider-item">
          <div class="provider-info">
            <span class="provider-name">{{ p.name }}</span>
            <span class="provider-model">{{ p.model_name }}</span>
            <span v-if="p.is_default" class="provider-default">{{ t('common.default') }}</span>
          </div>
          <div class="provider-actions">
            <button v-if="!p.is_default" @click="setDefault(p.id)">{{ t('ai.setDefault') }}</button>
            <button @click="handleEdit(p)">{{ t('common.edit') }}</button>
            <button class="danger" @click="handleDelete(p.id)">{{ t('common.delete') }}</button>
          </div>
        </div>
      </div>
      <button v-if="!showForm" class="btn-add" @click="openAddForm">{{ t('ai.addProvider') }}</button>
      <LlmProviderForm v-if="showForm" :initial-data="editingProvider" @save="handleSave" @cancel="showForm = false" />
    </div>

    <!-- Chat Area -->
    <template v-else>
      <ChatArea :messages="aiStore.messages" @apply="aiStore.applyResult($event)" @dismiss="aiStore.dismiss()" />
      <div v-if="aiStore.isProcessing" class="processing-bar">
        <span class="processing-text">{{ t('ai.processing') }}</span>
        <button class="cancel-btn" @click="aiStore.cancel()">{{ t('ai.cancel') }}</button>
      </div>
      <ChatInput :disabled="aiStore.isProcessing" @submit="(c: string, a: any) => aiStore.submitInput(c, a)" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAiStore } from '@/stores/ai';
import { useSettingsStore } from '@/stores/settings';
import { invoke } from '@tauri-apps/api/core';
import { confirm as tauriConfirm, message as tauriMessage } from '@tauri-apps/plugin-dialog';
import ChatArea from './ChatArea.vue';
import ChatInput from './ChatInput.vue';
import LlmProviderForm from '@/components/settings/LlmProviderForm.vue';

const { t } = useI18n();
const aiStore = useAiStore();
const settingsStore = useSettingsStore();
const showConfig = ref(false);
const showForm = ref(false);
const editingProvider = ref<any>(null);

onMounted(() => { settingsStore.loadProviders(); });

async function handleSave(form: any) {
  try {
    await settingsStore.saveProvider(form);
    showForm.value = false;
    editingProvider.value = null;
  } catch (e: any) {
    await tauriMessage(e.message || String(e), { title: t('ai.saveFailed'), kind: 'error' });
  }
}

function openAddForm() {
  editingProvider.value = null;
  showForm.value = true;
}

function handleEdit(p: any) {
  editingProvider.value = { ...p };
  showForm.value = true;
}

async function handleDelete(id: string) {
  const yes = await tauriConfirm(t('ai.deleteProviderConfirm'), { title: t('ai.deleteConfirmTitle'), kind: 'warning' });
  if (yes) {
    try { await settingsStore.deleteProvider(id); } catch (e: any) { await tauriMessage(e.message || String(e), { title: t('ai.deleteFailed'), kind: 'error' }); }
  }
}

async function setDefault(id: string) {
  try {
    const provider = settingsStore.providers.find(p => p.id === id);
    if (!provider) return;
    await invoke('save_provider', { config: { ...provider, is_default: true, api_key: '' } });
    for (const p of settingsStore.providers) {
      if (p.id !== id && p.is_default) {
        await invoke('save_provider', { config: { ...p, is_default: false, api_key: '' } });
      }
    }
    await settingsStore.loadProviders();
  } catch (e: any) {
    await tauriMessage(e.message || String(e), { title: t('ai.setDefaultFailed'), kind: 'error' });
  }
}
</script>

<style scoped>
.ai-sidebar {
  width: var(--ai-sidebar-width);
  min-width: var(--ai-sidebar-width);
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-left: 1px solid var(--border-color);
}
.sidebar-header {
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.header-actions { display: flex; gap: 4px; }
.header-actions button {
  background: none; border: none; cursor: pointer; font-size: 14px;
  color: var(--text-secondary); padding: 2px 6px; border-radius: 4px;
  display: flex; align-items: center;
}
.header-actions button:hover { background: var(--border-color); }
.header-actions button.active { color: var(--accent-color); background: var(--border-color); }

.config-panel {
  flex: 1; overflow-y: auto; padding: 12px;
}
.no-providers { text-align: center; color: var(--text-secondary); padding: 20px; }
.no-providers .hint { font-size: 12px; margin-top: 4px; opacity: 0.7; }
.provider-list { margin-bottom: 12px; }
.provider-item {
  padding: 8px 10px; border: 1px solid var(--border-color);
  border-radius: 6px; margin-bottom: 6px;
}
.provider-info { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; margin-bottom: 6px; }
.provider-name { font-weight: 600; font-size: 13px; }
.provider-model { font-size: 11px; color: var(--text-secondary); }
.provider-default {
  font-size: 10px; background: var(--accent-color); color: white;
  padding: 1px 5px; border-radius: 3px;
}
.provider-actions { display: flex; gap: 4px; }
.provider-actions button {
  padding: 3px 8px; font-size: 11px; background: none;
  border: 1px solid var(--border-color); border-radius: 4px; cursor: pointer;
  color: var(--text-primary);
}
.provider-actions button:hover { background: var(--bg-secondary); }
.provider-actions button.danger { border-color: var(--danger-color, red); color: var(--danger-color, red); }
.provider-actions button.danger:hover { background: rgba(255,0,0,0.05); }
.btn-add {
  width: 100%; padding: 8px; font-size: 13px; background: var(--accent-color);
  color: white; border: none; border-radius: 6px; cursor: pointer;
}
.btn-add:hover { opacity: 0.9; }

.processing-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 8px 12px; background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
}
.processing-text { font-size: 12px; color: var(--text-secondary); }
.cancel-btn {
  padding: 4px 12px; font-size: 12px; background: var(--danger-color);
  color: white; border: none; border-radius: 4px; cursor: pointer;
}
.cancel-btn:hover { opacity: 0.9; }
</style>
