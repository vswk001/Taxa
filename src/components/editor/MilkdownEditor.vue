<template>
  <MilkdownProvider>
    <MilkdownCore
      :model-value="modelValue"
      :note-id="noteId"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </MilkdownProvider>
</template>

<script setup lang="ts">
import { MilkdownProvider } from '@milkdown/vue';
import MilkdownCore from './MilkdownCore.vue';

defineProps<{ modelValue: string; noteId?: string }>();
defineEmits<{ 'update:modelValue': [value: string] }>();
</script>
