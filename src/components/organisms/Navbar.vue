<script setup lang="ts">
import { ref } from 'vue';
import { FolderGit2, GitBranch, RefreshCw, Settings, Sparkles } from '@lucide/vue';

const props = defineProps<{
  repoPath: string;
  branches: Array<{ name: string; is_head: boolean }>;
  currentBranch: string;
  activeProvider: string;
  activeModel: string;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:repoPath', path: string): void;
  (e: 'refresh'): void;
  (e: 'openSettings'): void;
}>();

const inputPath = ref(props.repoPath);

function handleOpenRepo() {
  if (inputPath.value.trim()) {
    emit('update:repoPath', inputPath.value.trim());
  }
}
</script>

<template>
  <header class="navbar glass-panel">
    <div class="brand">
      <div class="logo-icon">
        <Sparkles :size="20" class="sparkle-svg" />
      </div>
      <span class="brand-title">Branch<span class="highlight">AI</span></span>
    </div>

    <div class="repo-selector">
      <FolderGit2 :size="16" class="icon-muted" />
      <input v-model="inputPath" @keyup.enter="handleOpenRepo" placeholder="Enter git repository path..."
        class="repo-input" />
      <button @click="handleOpenRepo" class="btn btn-secondary btn-sm">
        Open Repo
      </button>
    </div>

    <div class="branch-status" v-if="currentBranch">
      <GitBranch :size="14" class="icon-branch" />
      <span class="branch-name">{{ currentBranch }}</span>
    </div>

    <div class="spacer"></div>

    <div class="ai-badge" @click="emit('openSettings')" title="Click to configure AI Providers">
      <Sparkles :size="14" class="ai-sparkle" />
      <span class="ai-provider-text">{{ activeProvider.toUpperCase() }} ({{ activeModel }})</span>
    </div>

    <button @click="emit('refresh')" class="icon-btn" :class="{ spinning: loading }" title="Refresh Repository">
      <RefreshCw :size="16" />
    </button>

    <button @click="emit('openSettings')" class="icon-btn" title="Settings & API Keys">
      <Settings :size="16" />
    </button>
  </header>
</template>

<style scoped src="../../styles/organisms/Navbar.css"></style>
