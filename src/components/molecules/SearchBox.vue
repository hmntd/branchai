<script setup lang="ts">
import { Search } from '@lucide/vue';

withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
  }>(),
  {
    placeholder: 'Filter...',
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', val: string): void;
}>();
</script>

<template>
  <div class="search-box-wrapper">
    <input :value="modelValue" @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      :placeholder="placeholder" class="search-box-input" />
    <Search :size="11" class="search-box-icon" />
  </div>
</template>

<style scoped src="../../styles/molecules/SearchBox.css"></style>
