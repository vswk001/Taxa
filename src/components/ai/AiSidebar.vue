<template>
  <div class="ai-sidebar">
    <div class="sidebar-header">
      <span>{{ t('ai.assistant') }}</span>
      <div class="header-actions">
        <button @click="runCheck" :disabled="aiStore.isProcessing" :title="t('ai.libraryCheck')">🩺</button>
        <button @click="openHistory" :title="t('ai.libraryHistory')">🕘</button>
        <button v-if="aiStore.messages.length > 0" @click="aiStore.clearMessages()" :title="t('ai.clearChat')">🗑</button>
        <button :class="{ active: showConfig }" @click="showConfig = !showConfig" :title="t('ai.llmConfig')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
        </button>
      </div>
    </div>

    <!-- LLM Config Panel; v-show keeps the chat (scroll position, state) alive -->
    <div v-show="showConfig" class="config-panel">
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
          <div class="provider-info">
            <span class="drag-handle" :title="t('ai.reorderHint')">⋮⋮</span>
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
    <div v-show="!showConfig" class="chat-wrap">
      <ChatArea
        :messages="aiStore.messages"
        @apply="onApply"
        @apply-optimize="onApplyOptimize"
        @dismiss="aiStore.dismiss($event)"
        @open-note="openNoteById"
        @lib-apply="aiStore.applyLibrarySuggestion($event.msgId, $event.index)"
        @lib-skip="aiStore.skipLibrarySuggestion($event.msgId, $event.index)"
      />
      <div v-if="aiStore.isProcessing" class="processing-bar">
        <span class="processing-text">{{ t('ai.processing') }}</span>
        <button class="cancel-btn" @click="aiStore.cancel()">{{ t('ai.cancel') }}</button>
      </div>
      <div class="input-area">
        <div class="mode-select-wrap">
          <select v-model="aiStore.mode" class="mode-select">
            <option value="organize">{{ t('ai.modeOrganize') }} — {{ t('ai.modeOrganizeDesc') }}</option>
            <option value="optimize">{{ t('ai.modeOptimize') }} — {{ t('ai.modeOptimizeDesc') }}</option>
            <option value="ask">{{ t('ai.modeAsk') }} — {{ t('ai.modeAskDesc') }}</option>
          </select>
        </div>
        <ChatInput :disabled="aiStore.isProcessing" :mode="aiStore.mode" @submit="handleSubmit" />
      </div>
    </div>
    <Teleport to="body">
      <div v-if="historyVisible" class="history-overlay" @click.self="historyVisible = false">
        <div class="history-dialog">
          <div class="history-header">
            <span>{{ t('ai.libraryHistory') }}</span>
            <button class="close-btn" @click="historyVisible = false">×</button>
          </div>
          <div class="history-list">
            <div v-if="!history.length" class="history-empty">{{ t('ai.historyEmpty') }}</div>
            <div v-for="op in history" :key="op.id" class="history-item">
              <span class="history-type">{{ opLabel(op.operation_type) }}</span>
              <span class="history-time">{{ op.created_at.slice(0, 16).replace('T', ' ') }}</span>
              <button
                v-if="op.operation_type.startsWith('library-')"
                class="undo-btn"
                @click="undoOp(op.id)"
              >{{ t('ai.undo') }}</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
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
import { ref, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { useAiStore } from '@/stores/ai';
import { useNotebookStore } from '@/stores/notebook';
import { useSettingsStore } from '@/stores/settings';
import { message as tauriMessage } from '@tauri-apps/plugin-dialog';
import type { LlmProvider, LlmProviderForm as LlmProviderFormType } from '@/types/settings';
import { invoke } from '@tauri-apps/api/core';
import { useProviderDrag } from '@/composables/useProviderDrag';
import ChatArea from './ChatArea.vue';
import ChatInput from './ChatInput.vue';
import LlmProviderForm from '@/components/settings/LlmProviderForm.vue';
import ConfirmDialog from '@/components/common/ConfirmDialog.vue';

const { t } = useI18n();
const aiStore = useAiStore();
const notebookStore = useNotebookStore();
const settingsStore = useSettingsStore();
const { providers: providersRef } = storeToRefs(settingsStore);
const { dragIndex, overIndex, onDragStart, onDragOver, onDrop, onDragEnd } =
  useProviderDrag(providersRef, (ids) => settingsStore.reorderProviders(ids));
const showConfig = ref(false);
const showForm = ref(false);
const editingProvider = ref<LlmProvider | null>(null);
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

async function openNoteById(id: string) {
  const notebookStore = useNotebookStore();
  await notebookStore.openNote(id);
  const note = notebookStore.currentNote;
  if (note) {
    const { useEditorStore } = await import('@/stores/editor');
    useEditorStore().openTab(note.note.id, note.note.title);
  }
}

async function runCheck() {
  await aiStore.runLibraryCheck();
}

interface HistoryOp {
  id: string;
  note_id: string | null;
  operation_type: string;
  after_state: string | null;
  created_at: string;
}

const historyVisible = ref(false);
const history = ref<HistoryOp[]>([]);

async function openHistory() {
  historyVisible.value = true;
  try {
    history.value = await invoke<HistoryOp[]>('list_ai_operations');
  } catch (e) {
    console.error('failed to list ai operations:', e);
  }
}

function opLabel(type: string): string {
  if (type === 'library-move') return t('ai.libMoveOp');
  if (type === 'library-tag') return t('ai.libTagOp');
  if (type === 'library-merge') return t('ai.libMergeOp');
  return t('ai.libOrganizeOp');
}

async function undoOp(id: string) {
  try {
    await invoke('undo_ai_operation', { opId: id });
    await openHistory();
    const nb = useNotebookStore();
    await nb.loadFolderTree();
    await nb.loadAllNotes();
  } catch (e) {
    console.error('undo failed:', e);
    const { message } = await import('@tauri-apps/plugin-dialog');
    await message(e instanceof Error ? e.message : String(e), { title: t('ai.undoFailed'), kind: 'error' });
  }
}

function onApply(payload: { result: import('@/types/ai-extended').OrganizeResult; msgId: string }) {
  aiStore.applyResult(payload.result, payload.msgId);
}

function onApplyOptimize(payload: { noteId: string; title: string; content: string; msgId: string }) {
  aiStore.applyOptimize(payload.noteId, payload.title, payload.content, payload.msgId);
}

function handleSubmit(content: string, attachments: import('@/types/ai').FileAttachment[]) {
  if (aiStore.mode === 'optimize') {
    const note = notebookStore.currentNote;
    if (!note) {
      aiStore.messages.push({
        id: crypto.randomUUID(),
        role: 'system',
        content: t('ai.noNoteForOptimize'),
        timestamp: new Date().toISOString(),
        status: 'error',
      });
      return;
    }
    aiStore.optimizeNote(note.note.id, content);
  } else if (aiStore.mode === 'ask') {
    aiStore.askNote(content);
  } else {
    aiStore.submitInput(content, attachments);
  }
}

function openAddForm() {
  editingProvider.value = null;
  showForm.value = true;
}

onMounted(() => { settingsStore.loadProviders(); });

async function handleSave(form: LlmProviderFormType & { id?: string }) {
  try {
    await settingsStore.saveProvider(form);
    showForm.value = false;
    editingProvider.value = null;
  } catch (e: unknown) {
    await tauriMessage(extractErr(e).message, { title: t('ai.saveFailed'), kind: 'error' });
  }
}

function extractErr(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e));
}

function handleEdit(p: LlmProvider) {
  editingProvider.value = { ...p };
  showForm.value = true;
}

async function handleDelete(id: string) {
  const yes = await showConfirm(t('ai.deleteProviderConfirm'));
  if (yes) {
    try { await settingsStore.deleteProvider(id); } catch (e: unknown) { await tauriMessage(extractErr(e).message, { title: t('ai.deleteFailed'), kind: 'error' }); }
  }
}

async function setDefault(id: string) {
  try {
    // Single store call: the backend unsets other defaults in one transaction.
    await settingsStore.setDefault(id);
  } catch (e: unknown) {
    await tauriMessage(extractErr(e).message, { title: t('ai.setDefaultFailed'), kind: 'error' });
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

.history-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  z-index: 210;
  display: flex;
  align-items: center;
  justify-content: center;
}

.history-dialog {
  width: 460px;
  max-height: 420px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.history-header {
  display: flex;
  justify-content: space-between;
  padding: 12px 16px;
  font-weight: 600;
  font-size: 14px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.history-header .close-btn {
  background: none; border: none; font-size: 18px; cursor: pointer; color: var(--text-secondary);
}

.history-list { flex: 1; overflow-y: auto; padding: 8px; }

.history-empty { padding: 30px; text-align: center; color: var(--text-secondary); font-size: 13px; }

.history-item {
  display: flex; align-items: center; gap: 8px;
  padding: 7px 10px; border-bottom: 1px solid var(--border-color); font-size: 12px;
}

.history-type { flex: 1; color: var(--text-primary); }

.history-time { color: var(--text-secondary); font-size: 11px; }

.undo-btn {
  padding: 2px 10px; font-size: 11px; border-radius: 4px;
  border: 1px solid var(--border-color); background: none; cursor: pointer; color: var(--text-primary);
}

.undo-btn:hover { border-color: var(--accent-color); color: var(--accent-color); }

.config-panel {
  flex: 1; overflow-y: auto; padding: 12px;
}
.chat-wrap {
  flex: 1; display: flex; flex-direction: column; min-height: 0;
}
.no-providers { text-align: center; color: var(--text-secondary); padding: 20px; }
.no-providers .hint { font-size: 12px; margin-top: 4px; opacity: 0.7; }
.provider-list { margin-bottom: 12px; }
.provider-item {
  padding: 8px 10px; border: 1px solid var(--border-color);
  border-radius: 6px; margin-bottom: 6px;
}
.provider-item.dragover { border-color: var(--accent-color); box-shadow: 0 0 0 1px var(--accent-color); }
.provider-item.dragging { opacity: 0.5; }
.drag-handle {
  cursor: grab; color: var(--text-secondary); opacity: 0.5;
  font-size: 13px; line-height: 1; letter-spacing: -2px; user-select: none;
}
.drag-handle:hover { opacity: 1; }
.reorder-hint { font-size: 11px; color: var(--text-secondary); margin-bottom: 6px; opacity: 0.8; }
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

.input-area { border-top: 1px solid var(--border-color); }
.mode-select-wrap { padding: 6px 12px 0; }
.mode-select {
  width: 100%; padding: 5px 8px; font-size: 12px;
  border: 1px solid var(--border-color); border-radius: 6px;
  background: var(--bg-primary); color: var(--text-primary);
  outline: none; cursor: pointer;
}
.mode-select:focus { border-color: var(--accent-color); }
</style>
