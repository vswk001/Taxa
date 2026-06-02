<template>
  <div class="settings-overlay" v-if="visible" @click.self="emit('close')">
    <div class="settings-dialog">
      <div class="settings-header">
        <h3>设置</h3>
        <button @click="emit('close')">×</button>
      </div>
      <div class="settings-tabs">
        <button :class="{ active: tab === 'llm' }" @click="tab = 'llm'">LLM 配置</button>
        <button :class="{ active: tab === 'theme' }" @click="tab = 'theme'">主题</button>
      </div>
      <div class="settings-body">
        <div v-if="tab === 'llm'">
          <div class="provider-list">
            <div v-for="p in settingsStore.providers" :key="p.id" class="provider-item">
              <div class="provider-info">
                <span class="provider-name">{{ p.name }}</span>
                <span class="provider-model">{{ p.model_name }}</span>
                <span v-if="p.is_default" class="provider-default">默认</span>
              </div>
              <button class="btn-delete" @click="handleDeleteProvider(p.id)">删除</button>
            </div>
          </div>
          <button v-if="!showForm" class="btn-add" @click="showForm = true">+ 添加 LLM 提供商</button>
          <LlmProviderForm v-if="showForm" @save="handleSave" @cancel="showForm = false" />
        </div>
        <div v-if="tab === 'theme'" class="theme-settings">
          <label class="theme-option">
            <input type="radio" value="system" v-model="localTheme" @change="handleThemeChange" />
            <span>跟随系统</span>
          </label>
          <label class="theme-option">
            <input type="radio" value="light" v-model="localTheme" @change="handleThemeChange" />
            <span>亮色</span>
          </label>
          <label class="theme-option">
            <input type="radio" value="dark" v-model="localTheme" @change="handleThemeChange" />
            <span>暗色</span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import LlmProviderForm from './LlmProviderForm.vue';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const settingsStore = useSettingsStore();
const tab = ref('llm');
const showForm = ref(false);
const localTheme = ref<'light' | 'dark' | 'system'>(settingsStore.theme);

onMounted(() => {
  settingsStore.loadProviders();
});

// Reset local theme when dialog opens
watch(() => props.visible, (visible) => {
  if (visible) {
    localTheme.value = settingsStore.theme;
  }
});

async function handleSave(form: any) {
  await settingsStore.saveProvider(form);
  showForm.value = false;
}

async function handleDeleteProvider(id: string) {
  if (confirm('确定要删除这个提供商吗?')) {
    await settingsStore.deleteProvider(id);
  }
}

function handleThemeChange() {
  const theme = localTheme.value;
  applyTheme(theme);
  // In a real app, you'd save this to persistent storage
  // settingsStore.theme = theme;
}

function applyTheme(theme: 'light' | 'dark' | 'system') {
  if (theme === 'dark') {
    document.documentElement.setAttribute('data-theme', 'dark');
  } else if (theme === 'light') {
    document.documentElement.removeAttribute('data-theme');
  } else {
    // System preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (prefersDark) {
      document.documentElement.setAttribute('data-theme', 'dark');
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
  }
}
</script>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.4);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}
.settings-dialog {
  width: 600px;
  max-height: 80vh;
  background: var(--bg-primary);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-color);
}
.settings-header h3 {
  font-size: 16px;
  margin: 0;
}
.settings-header button {
  font-size: 24px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.settings-header button:hover {
  color: var(--text-primary);
}
.settings-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
}
.settings-tabs button {
  flex: 1;
  padding: 10px;
  font-size: 13px;
  border: none;
  background: none;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
}
.settings-tabs button:hover {
  color: var(--text-primary);
}
.settings-tabs button.active {
  border-bottom-color: var(--accent-color);
  color: var(--accent-color);
}
.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.provider-list {
  margin-bottom: 16px;
}
.provider-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  margin-bottom: 8px;
}
.provider-info {
  display: flex;
  gap: 8px;
  align-items: center;
}
.provider-name {
  font-weight: 600;
  font-size: 13px;
}
.provider-model {
  font-size: 12px;
  color: var(--text-secondary);
}
.provider-default {
  font-size: 11px;
  background: var(--accent-color);
  color: white;
  padding: 2px 6px;
  border-radius: 3px;
}
.btn-delete {
  padding: 4px 8px;
  font-size: 12px;
  background: var(--danger-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}
.btn-delete:hover {
  opacity: 0.9;
}
.btn-add {
  width: 100%;
  padding: 8px;
  font-size: 13px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: var(--radius);
  cursor: pointer;
}
.btn-add:hover {
  opacity: 0.9;
}
.theme-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.theme-option {
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}
.theme-option input {
  cursor: pointer;
}
.theme-option span {
  cursor: pointer;
}
</style>
