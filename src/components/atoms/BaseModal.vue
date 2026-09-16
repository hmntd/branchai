<script setup lang="ts">
import { X } from '@lucide/vue';

withDefaults(
  defineProps<{
    title?: string;
    size?: 'normal' | 'wide' | 'extra-wide';
  }>(),
  {
    title: '',
    size: 'normal',
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
}>();
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-card" :class="size !== 'normal' ? size : ''">
      <div class="modal-header">
        <slot name="header-icon" />
        <span class="title">{{ title }}</span>
        <button class="modal-close-btn" @click="emit('close')">
          <X :size="16" />
        </button>
      </div>

      <div class="modal-body">
        <slot />
      </div>

      <div v-if="$slots.footer" class="modal-footer">
        <slot name="footer" />
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/atoms/BaseModal.css"></style>
