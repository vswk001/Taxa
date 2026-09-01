<template>
  <div class="settings-overlay" v-if="visible" @click.self="emit('close')">
    <div class="settings-dialog">
      <div class="settings-sidebar">
        <div class="settings-title">{{ t('settings.title') }}</div>
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="settings-tab"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          <span class="tab-icon">{{ tab.icon }}</span>
          <span class="tab-label">{{ t(tab.labelKey) }}</span>
        </button>
      </div>
      <div class="settings-main">
        <div class="settings-header">
          <span class="settings-header-title">{{ currentTabLabel }}</span>
          <button class="settings-close" @click="emit('close')">×</button>
        </div>
        <div class="settings-body">
          <!-- 通用 -->
          <div v-if="activeTab === 'general'" class="settings-section">
            <div class="section-title">{{ t('settings.appearance') }}</div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.theme') }}</span>
              <div class="theme-options">
                <label class="theme-option" :class="{ active: localTheme === 'system' }">
                  <input type="radio" value="system" v-model="localTheme" @change="handleThemeChange" />
                  <span>{{ t('settings.themeSystem') }}</span>
                </label>
                <label class="theme-option" :class="{ active: localTheme === 'light' }">
                  <input type="radio" value="light" v-model="localTheme" @change="handleThemeChange" />
                  <span>{{ t('settings.themeLight') }}</span>
                </label>
                <label class="theme-option" :class="{ active: localTheme === 'dark' }">
                  <input type="radio" value="dark" v-model="localTheme" @change="handleThemeChange" />
                  <span>{{ t('settings.themeDark') }}</span>
                </label>
              </div>
            </div>
            <div class="section-title" style="margin-top: 24px;">{{ t('settings.language') }}</div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.language') }}</span>
              <select class="lang-select" v-model="localLang" @change="handleLangChange">
                <option v-for="loc in supportedLocales" :key="loc.value" :value="loc.value">{{ loc.label }}</option>
              </select>
            </div>
            <div class="section-title" style="margin-top: 24px;">{{ t('settings.quickCapture') }}</div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.quickCaptureEnable') }}</span>
              <input type="checkbox" v-model="quickCapture.enabled" @change="saveQuickCapture" />
            </div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.quickCaptureShortcut') }}</span>
              <input
                class="shortcut-input"
                v-model="quickCapture.accelerator"
                :disabled="!quickCapture.enabled"
                :placeholder="defaultAccelerator"
                @change="saveQuickCapture"
              />
            </div>
            <p class="setting-hint">{{ t('settings.quickCaptureHint') }}</p>
            <div class="section-title" style="margin-top: 24px;">{{ t('settings.traySection') }}</div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.closeToTray') }}</span>
              <input type="checkbox" v-model="closeToTray" @change="saveCloseToTray" />
            </div>
            <p class="setting-hint">{{ t('settings.closeToTrayHint') }}</p>
            <div class="section-title" style="margin-top: 24px;">{{ t('settings.dataSection') }}</div>
            <div class="setting-item">
              <span class="setting-label">{{ t('settings.backupLabel') }}</span>
              <div class="backup-actions">
                <button class="backup-btn" :disabled="busyAction === 'backup'" @click="doBackup">
                  {{ busyAction === 'backup' ? t('settings.backupWorking') : t('settings.backupNow') }}
                </button>
                <button class="backup-btn" :disabled="busyAction === 'restore'" @click="doRestore">
                  {{ t('settings.restoreBackup') }}
                </button>
              </div>
            </div>
            <p class="setting-hint">{{ t('settings.backupHint') }}</p>
          </div>

          <!-- LLM 配置 -->
          <div v-if="activeTab === 'llm'" class="settings-section">
            <div class="section-title">{{ t('settings.llm') }}</div>
            <div v-if="settingsStore.providers.length === 0 && !showForm" class="no-providers">
              <p>{{ t('ai.noProviders') }}</p>
              <p class="hint">{{ t('ai.noProvidersHint') }}</p>
            </div>
            <div v-if="!showForm && settingsStore.providers.length > 1" class="reorder-hint">{{ t('ai.reorderHint') }}</div>
            <div v-if="!showForm" class="provider-list">
              <div
                v-for="(p, index) in settingsStore.providers"
                :key="p.id"
                class="provider-item draggable"
                :class="{ dragover: overIndex === index, dragging: dragIndex === index }"
                draggable="true"
                @dragstart="onDragStart(index)"
                @dragover="onDragOver($event, index)"
                @drop="onDrop(index)"
                @dragend="onDragEnd"
              >
                <span class="drag-handle" :title="t('ai.reorderHint')">⋮⋮</span>
                <div class="provider-info">
                  <span class="provider-name">{{ p.name }}</span>
                  <span class="provider-type-badge">{{ p.provider_type }}</span>
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
            <button v-if="!showForm" class="btn-add" @click="openAddForm">{{ t('ai.addProviderFull') }}</button>
            <LlmProviderForm v-if="showForm" :initial-data="editingProvider" @save="handleSave" @cancel="showForm = false" />
          </div>

          <!-- 关于 -->
          <div v-if="activeTab === 'about'" class="settings-section">
            <div class="about-content">
              <div class="about-title">{{ t('settings.aboutTitle') }}</div>
              <div class="about-desc">{{ t('settings.aboutDesc') }}</div>
              <div class="about-version">{{ t('settings.aboutVersion', { version: appVersion }) }}</div>
              <div class="about-tech">{{ t('settings.aboutTech') }}</div>
              <div class="updater">
                <button
                  class="check-update-btn"
                  :disabled="['checking','downloading','installing'].includes(updater.state.value)"
                  @click="updater.checkForUpdates"
                >{{ t('settings.checkUpdate') }}</button>
                <span class="update-status">
                  <template v-if="updater.state.value === 'checking'">{{ t('settings.updateChecking') }}</template>
                  <template v-else-if="updater.state.value === 'uptodate'">{{ t('settings.updateUptodate') }}</template>
                  <template v-else-if="updater.state.value === 'available'">
                    {{ t('settings.updateAvailable', { version: updater.newVersion.value }) }}
                    <button class="install-btn" :disabled="false" @click="updater.downloadAndInstall">{{ t('settings.updateInstall') }}</button>
                  </template>
                  <template v-else-if="updater.state.value === 'downloading'">{{ t('settings.updateDownloading', { p: updater.progress.value }) }}</template>
                  <template v-else-if="updater.state.value === 'installing'">{{ t('settings.updateInstalling') }}</template>
                  <template v-else-if="updater.state.value === 'error'">{{ t('settings.updateError') }}: {{ updater.errorMsg.value }}</template>
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <ConfirmDialog
      :visible="confirmVisible"
      :message="confirmMsg"
      kind="danger"
      @confirm="confirmResolve?.(true); confirmVisible = false"
      @cancel="confirmResolve?.(false); confirmVisible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { useSettingsStore } from '@/stores/settings';
import { invoke } from '@tauri-apps/api/core';
import { message as tauriMessage } from '@tauri-apps/plugin-dialog';
import { setLocale, SUPPORTED_LOCALES } from '@/i18n';
import { getVersion } from '@tauri-apps/api/app';
import { useTheme } from '@/composables/useTheme';
import {
  loadQuickCaptureSettings,
  saveQuickCaptureSettings,
  applyQuickCaptureShortcut,
  DEFAULT_ACCELERATOR,
} from '@/composables/useQuickCapture';
import type { LlmProvider } from '@/types/settings';
import { useProviderDrag } from '@/composables/useProviderDrag';
import { useUpdater } from '@/composables/useUpdater';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';
import LlmProviderForm from './LlmProviderForm.vue';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ close: [] }>();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const { providers: providersRef } = storeToRefs(settingsStore);
const { dragIndex, overIndex, onDragStart, onDragOver, onDrop, onDragEnd } =
  useProviderDrag(providersRef, (ids) => settingsStore.reorderProviders(ids));
const updater = useUpdater();
const { setTheme } = useTheme();

// Real app version from the Tauri bundle (previously hard-coded in each
// locale file and already drifting from the actual release).
const appVersion = ref('');
onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch {
    appVersion.value = '0.4.2';
  }
});

const activeTab = ref('general');
const quickCapture = ref(loadQuickCaptureSettings());
const closeToTray = ref((localStorage.getItem('taxa-close-to-tray') ?? 'true') === 'true');
const busyAction = ref<'' | 'backup' | 'restore'>('');
const defaultAccelerator = DEFAULT_ACCELERATOR;
const showForm = ref(false);
const editingProvider = ref<LlmProvider | null>(null);
const localTheme = ref<'light' | 'dark' | 'system'>(settingsStore.theme);
const localLang = ref((localStorage.getItem('taxa-locale') || 'en'));
const supportedLocales = SUPPORTED_LOCALES;
const confirmVisible = ref(false);
const confirmMsg = ref('');
const confirmResolve = ref<((v: boolean) => void) | null>(null);

function showConfirm(msg: string): Promise<boolean> {
  return new Promise((resolve) => {
    confirmMsg.value = msg;
    confirmVisible.value = true;
    confirmResolve.value = resolve;
  });
}

const tabs = [
  { key: 'general', icon: '⚙️', labelKey: 'settings.general' },
  { key: 'llm', icon: '🤖', labelKey: 'settings.llm' },
  { key: 'about', icon: 'ℹ️', labelKey: 'settings.about' },
];

const currentTabLabel = computed(() => {
  const tab = tabs.find(tb => tb.key === activeTab.value);
  return tab ? t(tab.labelKey) : '';
});

onMounted(() => { settingsStore.loadProviders(); });

watch(() => props.visible, (v) => {
  if (v) {
    localTheme.value = settingsStore.theme;
    localLang.value = localStorage.getItem('taxa-locale') || 'en';
    showForm.value = false;
    editingProvider.value = null;
    settingsStore.loadProviders();
  }
});

function handleThemeChange() {
  setTheme(localTheme.value);
}

async function saveCloseToTray() {
  localStorage.setItem('taxa-close-to-tray', String(closeToTray.value));
  try {
    await invoke('set_close_to_tray', { enabled: closeToTray.value });
  } catch (e) {
    console.error('failed to set close-to-tray:', e);
  }
}

async function doBackup() {
  busyAction.value = 'backup';
  try {
    const done = await invoke<boolean>('backup_vault');
    if (done) await tauriMessage(t('settings.backupDone'), { kind: 'info' });
  } catch (e) {
    await tauriMessage(e instanceof Error ? e.message : String(e), { title: t('settings.backupFailed'), kind: 'error' });
  } finally {
    busyAction.value = '';
  }
}

async function doRestore() {
  const yes = await showConfirm(t('settings.restoreConfirm'));
  if (!yes) return;
  busyAction.value = 'restore';
  try {
    const staged = await invoke<boolean>('restore_vault');
    if (staged) {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    }
  } catch (e) {
    await tauriMessage(e instanceof Error ? e.message : String(e), { title: t('settings.backupFailed'), kind: 'error' });
    busyAction.value = '';
  }
}

async function saveQuickCapture() {
  saveQuickCaptureSettings(quickCapture.value);
  try {
    await applyQuickCaptureShortcut(quickCapture.value);
  } catch (e) {
    console.error('failed to apply quick-capture shortcut:', e);
  }
}

function handleLangChange() {
  setLocale(localLang.value);
}

async function handleSave(form: import('@/types/settings').LlmProviderForm & { id?: string }) {
  try {
    await settingsStore.saveProvider(form);
    showForm.value = false;
    editingProvider.value = null;
  } catch (e: unknown) {
    await tauriMessage(e instanceof Error ? e.message : String(e), { title: t('ai.saveFailed'), kind: 'error' });
  }
}

function openAddForm() {
  editingProvider.value = null;
  showForm.value = true;
}

function handleEdit(p: LlmProvider) {
  editingProvider.value = { ...p };
  showForm.value = true;
}

async function handleDelete(id: string) {
  const yes = await showConfirm(t('ai.deleteProviderConfirm'));
  if (yes) {
    try { await settingsStore.deleteProvider(id); } catch (e: unknown) { await tauriMessage(e instanceof Error ? e.message : String(e), { title: t('ai.deleteFailed'), kind: 'error' }); }
  }
}

async function setDefault(id: string) {
  try {
    await settingsStore.setDefault(id);
  } catch (e: unknown) {
    await tauriMessage(e instanceof Error ? e.message : String(e), { title: t('ai.setDefaultFailed'), kind: 'error' });
  }
}
</script>

<style scoped>
.settings-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  z-index: 200; display: flex; align-items: center; justify-content: center;
}
.settings-dialog {
  width: 800px; height: 560px; background: var(--bg-primary);
  border-radius: 12px; overflow: hidden; display: flex;
}
.settings-sidebar {
  width: 160px; min-width: 160px; background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex; flex-direction: column; padding: 12px 0;
}
.settings-title {
  padding: 8px 16px 16px; font-weight: 600; font-size: 14px; color: var(--text-primary);
}
.settings-tab {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 16px; font-size: 13px; border: none; background: none;
  cursor: pointer; color: var(--text-secondary); text-align: left; width: 100%;
  border-left: 2px solid transparent;
}
.settings-tab:hover { background: var(--bg-primary); color: var(--text-primary); }
.settings-tab.active {
  color: var(--accent-color); background: var(--bg-primary);
  border-left-color: var(--accent-color);
}
.tab-icon { font-size: 14px; }
.tab-label { flex: 1; }

.settings-main {
  flex: 1; display: flex; flex-direction: column; min-width: 0;
}
.settings-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 14px 18px; border-bottom: 1px solid var(--border-color);
}
.settings-header-title { font-weight: 600; font-size: 14px; }
.settings-close {
  font-size: 24px; background: none; border: none; cursor: pointer;
  color: var(--text-secondary); padding: 0; width: 30px; height: 30px;
  display: flex; align-items: center; justify-content: center; border-radius: 4px;
}
.settings-close:hover { background: var(--bg-secondary); color: var(--text-primary); }

.settings-body { flex: 1; overflow-y: auto; padding: 20px; }

.settings-section { }
.section-title {
  font-weight: 600; font-size: 14px; color: var(--text-primary);
  margin-bottom: 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color);
}

.setting-item { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
.setting-label { font-size: 13px; color: var(--text-primary); }
.theme-options { display: flex; gap: 8px; }
.theme-option {
  display: flex; align-items: center; gap: 6px; padding: 6px 14px;
  border: 1px solid var(--border-color); border-radius: 6px;
  cursor: pointer; font-size: 13px; color: var(--text-secondary);
  transition: all 0.15s;
}
.theme-option:hover { border-color: var(--accent-color); }
.theme-option.active { border-color: var(--accent-color); color: var(--accent-color); background: rgba(74,144,217,0.08); }
.theme-option input { display: none; }

.shortcut-input {
  padding: 4px 8px;
  font-size: 13px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  width: 160px;
}

.shortcut-input:disabled {
  opacity: 0.5;
}

.backup-actions {
  display: flex;
  gap: 8px;
}

.backup-btn {
  padding: 5px 12px;
  font-size: 13px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  cursor: pointer;
  color: var(--text-primary);
}

.backup-btn:hover:not(:disabled) {
  border-color: var(--accent-color);
  color: var(--accent-color);
}

.backup-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.setting-hint {
  font-size: 11px;
  color: var(--text-secondary);
  margin: 4px 0 0;
  opacity: 0.8;
}

.lang-select {
  padding: 6px 12px; border: 1px solid var(--border-color); border-radius: 6px;
  font-size: 13px; background: var(--bg-secondary); color: var(--text-primary);
  cursor: pointer; min-width: 160px;
}
.lang-select:focus { outline: none; border-color: var(--accent-color); }

.no-providers { text-align: center; color: var(--text-secondary); padding: 20px; }
.no-providers .hint { font-size: 12px; margin-top: 4px; opacity: 0.7; }
.provider-list { margin-bottom: 16px; }
.provider-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 12px; border: 1px solid var(--border-color);
  border-radius: 6px; margin-bottom: 8px;
}
.provider-item.draggable { cursor: default; }
.provider-item.dragover { border-color: var(--accent-color); box-shadow: 0 0 0 1px var(--accent-color); }
.provider-item.dragging { opacity: 0.5; }
.drag-handle {
  cursor: grab; color: var(--text-secondary); opacity: 0.5;
  font-size: 14px; line-height: 1; letter-spacing: -2px; user-select: none;
}
.drag-handle:hover { opacity: 1; }
.reorder-hint { font-size: 11px; color: var(--text-secondary); margin-bottom: 6px; opacity: 0.8; }
.provider-info { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.provider-name { font-weight: 600; font-size: 13px; }
.provider-type-badge {
  font-size: 11px; background: var(--bg-secondary); padding: 2px 8px;
  border-radius: 10px; color: var(--text-secondary);
}
.provider-model { font-size: 12px; color: var(--text-secondary); }
.provider-default {
  font-size: 11px; background: var(--accent-color); color: white;
  padding: 2px 6px; border-radius: 3px;
}
.provider-actions { display: flex; gap: 6px; }
.provider-actions button {
  padding: 4px 8px; font-size: 11px; background: none;
  border: 1px solid var(--border-color); border-radius: 4px; cursor: pointer; color: var(--text-primary);
}
.provider-actions button:hover { background: var(--bg-secondary); }
.provider-actions button.danger { border-color: var(--danger-color, red); color: var(--danger-color, red); }
.provider-actions button.danger:hover { background: rgba(255,0,0,0.05); }
.btn-add {
  width: 100%; padding: 10px; font-size: 13px; background: var(--accent-color);
  color: white; border: none; border-radius: 6px; cursor: pointer;
}
.btn-add:hover { opacity: 0.9; }

.about-content { text-align: center; padding: 40px 20px; }
.about-title { font-size: 24px; font-weight: 700; color: var(--text-primary); }
.about-desc { font-size: 14px; color: var(--text-secondary); margin-top: 8px; }
.about-version { font-size: 13px; color: var(--text-secondary); margin-top: 16px; }
.about-tech { font-size: 12px; color: var(--text-secondary); opacity: 0.7; margin-top: 4px; }
.updater { margin-top: 28px; display: flex; flex-direction: column; align-items: center; gap: 10px; }
.check-update-btn { padding: 7px 18px; font-size: 13px; background: var(--accent-color); color: white; border: none; border-radius: 6px; cursor: pointer; }
.check-update-btn:hover:not(:disabled) { opacity: 0.9; }
.check-update-btn:disabled { opacity: 0.5; cursor: default; }
.update-status { font-size: 12px; color: var(--text-secondary); }
.update-status .install-btn { margin-left: 8px; padding: 3px 12px; font-size: 12px; background: var(--accent-color); color: white; border: none; border-radius: 4px; cursor: pointer; }
</style>
