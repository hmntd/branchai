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
      <input
        v-model="inputPath"
        @keyup.enter="handleOpenRepo"
        placeholder="Enter git repository path..."
        class="repo-input"
      />
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

<style scoped>
.navbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 20px;
  height: 56px;
  border-radius: 0;
  border-bottom: 1px solid var(--border-color);
  background: rgba(13, 17, 26, 0.95);
  z-index: 50;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;

}

.logo-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #a855f7 0%, #38bdf8 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.brand-title {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.highlight {
  color: var(--primary);
}

.repo-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.4);
  padding: 4px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  flex: 0 1 450px;
}

.repo-input {
  border: none;
  background: transparent;
  width: 100%;
  padding: 4px;
  font-family: var(--font-mono);
  font-size: 12px;
}
.repo-input:focus {
  box-shadow: none;
}

.branch-status {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 14px;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  color: var(--primary);
  font-size: 12px;
  font-weight: 600;
}

.spacer {
  flex: 1;
}

.ai-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 14px;
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.15), rgba(99, 102, 241, 0.15));
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: #c084fc;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.ai-badge:hover {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.25), rgba(99, 102, 241, 0.25));
  border-color: rgba(168, 85, 247, 0.5);
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.icon-btn:hover {
  color: var(--text-main);
  background: rgba(255, 255, 255, 0.08);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}
</style>
