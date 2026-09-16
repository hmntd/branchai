<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ai' | 'danger' | 'ghost';
    size?: 'xs' | 'sm' | 'md' | 'lg';
    disabled?: boolean;
    active?: boolean;
    iconOnly?: boolean;
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    disabled: false,
    active: false,
    iconOnly: false,
  }
);
</script>

<template>
  <button class="btn" :class="[
    `btn-${variant}`,
    size !== 'md' ? `btn-${size}` : '',
    { active, 'btn-icon-only': iconOnly }
  ]" :disabled="disabled">
    <slot />
  </button>
</template>

<style scoped src="../../styles/atoms/BaseButton.css"></style>
