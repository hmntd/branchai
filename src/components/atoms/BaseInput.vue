<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    label?: string;
    placeholder?: string;
    type?: string;
    mono?: boolean;
    size?: 'sm' | 'md';
  }>(),
  {
    label: '',
    placeholder: '',
    type: 'text',
    mono: false,
    size: 'md',
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'enter'): void;
}>();
</script>

<template>
  <div class="base-input-wrapper">
    <label v-if="label" class="base-input-label">{{ label }}</label>
    <input :type="type" :value="modelValue" :placeholder="placeholder" class="base-input" :class="[{ mono }, size]"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)" @keyup.enter="emit('enter')" />
  </div>
</template>

<style scoped src="../../styles/atoms/BaseInput.css"></style>
