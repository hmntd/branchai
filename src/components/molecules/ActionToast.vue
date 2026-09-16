<script setup lang="ts">
import { Check, AlertCircle } from '@lucide/vue';

defineProps<{
  message: string;
  isError?: boolean;
}>();
</script>

<template>
  <div v-if="message" class="action-toast" :class="isError ? 'toast-error' : 'toast-success'">
    <AlertCircle v-if="isError" :size="12" />
    <Check v-else :size="12" />
    <span>{{ message }}</span>
  </div>
</template>

<style scoped src="../../styles/molecules/ActionToast.css"></style>
