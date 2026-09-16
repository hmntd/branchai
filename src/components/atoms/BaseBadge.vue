<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'branch' | 'staged' | 'modified' | 'danger' | 'purple';
  }>(),
  {
    variant: 'branch',
  }
);
</script>

<template>
  <span class="badge" :class="`badge-${variant}`">
    <slot />
  </span>
</template>

<style scoped src="../../styles/atoms/BaseBadge.css"></style>
