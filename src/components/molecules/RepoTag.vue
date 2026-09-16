<script setup lang="ts">
defineProps<{
  path: string;
  active?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', path: string): void;
}>();
</script>

<template>
  <div class="repo-tag-wrapper">
    <span class="repo-tag" :class="{ active }" :title="path" @click="emit('select', path)">
      {{ path.split('/').pop() || path }}
    </span>
    <div class="tooltip-box">{{ path }}</div>
  </div>
</template>

<style scoped src="../../styles/molecules/RepoTag.css"></style>
