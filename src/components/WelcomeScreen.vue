<script setup lang="ts">
import {
  FolderGit2,
  Sparkles,
  ArrowRight,
  Clock,
  Settings,
  ShieldCheck,
  FolderPlus,
  X,
  Compass,
} from '@lucide/vue';

const props = defineProps<{
  recentRepos: string[];
  activeProvider: string;
  activeModel: string;
}>();

const emit = defineEmits<{
  (e: 'browse'): void;
  (e: 'selectRepo', path: string): void;
  (e: 'removeRecent', path: string): void;
  (e: 'openSettings'): void;
}>();
</script>

<template>
  <div class="welcome-container">
    <div class="welcome-card gk-panel">
      <!-- Glow effect top border -->
      <div class="card-glow"></div>

      <!-- Hero Header -->
      <div class="hero-header">
        <div class="brand-badge-wrap">
          <img src="/logo.png" alt="BranchAI Logo" class="welcome-logo-img" />
        </div>
        <h1 class="welcome-title">Welcome to BranchAI</h1>
        <p class="welcome-subtitle">
          Modern Native Git Client powered by AI (<span class="provider-pill">{{ activeProvider.toUpperCase() }} / {{ activeModel }}</span>)
        </p>
      </div>

      <!-- Main Action Section -->
      <div class="main-action-box">
        <button class="btn btn-primary btn-hero" @click="emit('browse')">
          <FolderPlus :size="18" />
          <span>Browse Repository</span>
          <ArrowRight :size="16" />
        </button>
        <p class="action-hint">Select a local Git repository folder from your system</p>
      </div>

      <!-- Recent Repositories Section -->
      <div class="recent-section" v-if="recentRepos.length > 0">
        <div class="section-title">
          <Clock :size="13" class="icon-muted" />
          <span>RECENT REPOSITORIES</span>
        </div>

        <div class="recent-grid">
          <div
            v-for="path in recentRepos"
            :key="path"
            class="recent-card"
            @click="emit('selectRepo', path)"
          >
            <div class="card-icon">
              <FolderGit2 :size="16" class="icon-primary" />
            </div>
            <div class="card-info">
              <span class="repo-name">{{ path.split('/').pop() || path }}</span>
              <span class="repo-path" :title="path">{{ path }}</span>
            </div>
            <button
              class="remove-btn"
              @click.stop="emit('removeRecent', path)"
              title="Remove from recent list"
            >
              <X :size="12" />
            </button>
          </div>
        </div>
      </div>

      <!-- Feature Grid -->
      <div class="feature-grid">
        <div class="feature-card">
          <Sparkles :size="16" class="icon-primary" />
          <span class="feature-title">AI Commit Generator</span>
          <p class="feature-desc">Automatically generates conventional commit messages from git diffs.</p>
        </div>

        <div class="feature-card">
          <ShieldCheck :size="16" class="icon-success" />
          <span class="feature-title">Smart Code Review</span>
          <p class="feature-desc">Analyzes diffs for bugs, security risks, and style improvements.</p>
        </div>

        <div class="feature-card">
          <Compass :size="16" class="icon-purple" />
          <span class="feature-title">Multi-Branch DAG Graph</span>
          <p class="feature-desc">High performance graphical visualizer with double-click branch switching.</p>
        </div>
      </div>

      <!-- Footer Quick Links -->
      <div class="welcome-footer">
        <button class="btn btn-secondary btn-xs" @click="emit('openSettings')">
          <Settings :size="12" />
          <span>Configure API Keys & Prompts</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.welcome-container {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  height: 100vh;
  width: 100vw;
  background: var(--bg-dark);
  padding: 20px;
  overflow-y: auto;
  user-select: none;
}

.welcome-card {
  width: 640px;
  max-width: 90vw;
  background: #181a1f;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  gap: 24px;
  position: relative;
  overflow: hidden;
}

.card-glow {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #00d2d3, #9c88ff, #2ed573);
}

.hero-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 8px;
}

.brand-badge-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.welcome-logo-img {
  width: 72px;
  height: 72px;
  border-radius: 16px;
  box-shadow: 0 8px 25px rgba(0, 210, 211, 0.35);
  border: 1px solid rgba(0, 210, 211, 0.4);
}

.icon-sparkles {
  color: #9c88ff;
}

.welcome-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-main);
  letter-spacing: -0.5px;
  margin: 0;
}

.welcome-subtitle {
  font-size: 13px;
  color: var(--text-muted);
  margin: 0;
}

.provider-pill {
  color: #9c88ff;
  font-weight: 600;
}

.main-action-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.05);
  padding: 20px;
  border-radius: 10px;
}

.btn-hero {
  padding: 10px 24px;
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 10px;
  border-radius: 8px;
  box-shadow: 0 4px 15px rgba(0, 210, 211, 0.3);
}

.action-hint {
  font-size: 11px;
  color: var(--text-dim);
  margin: 0;
}

.recent-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.recent-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 180px;
  overflow-y: auto;
}

.recent-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.recent-card:hover {
  background: rgba(0, 210, 211, 0.1);
  border-color: rgba(0, 210, 211, 0.3);
}

.card-icon {
  display: flex;
  align-items: center;
}

.card-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.repo-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-main);
}

.repo-path {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-btn {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 4px;
  border-radius: 3px;
  opacity: 0.6;
}

.remove-btn:hover {
  color: var(--danger);
  opacity: 1;
  background: rgba(255, 71, 87, 0.15);
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.feature-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 8px;
}

.feature-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-main);
}

.feature-desc {
  font-size: 10px;
  color: var(--text-dim);
  margin: 0;
  line-height: 1.3;
}

.welcome-footer {
  display: flex;
  justify-content: center;
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
}

.icon-purple { color: #9c88ff; }
</style>
