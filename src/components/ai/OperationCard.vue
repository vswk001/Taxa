<!-- src/components/ai/OperationCard.vue -->
<template>
  <div class="operation-card">
    <div class="card-title">AI 建议操作</div>
    <div class="card-body">
      <div class="field"><span class="label">操作：</span>{{ suggestion.action === 'create' ? '新建笔记' : '追加到已有笔记' }}</div>
      <div class="field"><span class="label">标题：</span>{{ suggestion.title }}</div>
      <div class="field"><span class="label">归类：</span>{{ suggestion.folder }}</div>
      <div class="field"><span class="label">标签：</span>{{ suggestion.tags?.join(', ') }}</div>
    </div>
    <div class="card-actions">
      <button class="btn-confirm" @click="emit('confirm')">确认</button>
      <button class="btn-dismiss" @click="emit('dismiss')">忽略</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AiSuggestion } from '@/types/ai';

defineProps<{ suggestion: AiSuggestion }>();
const emit = defineEmits<{ confirm: []; dismiss: [] }>();
</script>

<style scoped>
.operation-card {
  margin-top: 8px; border: 1px solid var(--border-color); border-radius: 8px;
  overflow: hidden; font-size: 12px;
}
.card-title { background: var(--accent-color); color: white; padding: 6px 10px; font-weight: 600; }
.card-body { padding: 8px 10px; }
.field { padding: 2px 0; }
.label { color: var(--text-secondary); }
.card-actions { display: flex; gap: 8px; padding: 8px 10px; border-top: 1px solid var(--border-color); }
.btn-confirm { flex: 1; padding: 6px; background: var(--success-color); color: white; border-radius: 4px; font-size: 12px; }
.btn-dismiss { flex: 1; padding: 6px; background: var(--bg-secondary); color: var(--text-secondary); border-radius: 4px; font-size: 12px; }
</style>
