<template>
  <MilkdownProvider>
    <MilkdownCore
      :model-value="modelValue"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </MilkdownProvider>
</template>

<script setup lang="ts">
import { MilkdownProvider } from '@milkdown/vue';
import MilkdownCore from './MilkdownCore.vue';

defineProps<{ modelValue: string }>();
defineEmits<{ 'update:modelValue': [value: string] }>();
</script>
