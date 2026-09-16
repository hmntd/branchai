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
import BaseButton from '../atoms/BaseButton.vue';

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
          Modern Native Git Client powered by AI (<span class="provider-pill">{{ activeProvider.toUpperCase() }} / {{
            activeModel }}</span>)
        </p>
      </div>

      <!-- Main Action Section -->
      <div class="main-action-box">
        <BaseButton variant="primary" class="btn-hero" @click="emit('browse')">
          <FolderPlus :size="18" />
          <span>Browse Repository</span>
          <ArrowRight :size="16" />
        </BaseButton>
        <p class="action-hint">Select a local Git repository folder from your system</p>
      </div>

      <!-- Recent Repositories Section -->
      <div class="recent-section" v-if="recentRepos.length > 0">
        <div class="section-title">
          <Clock :size="13" class="icon-muted" />
          <span>RECENT REPOSITORIES</span>
        </div>

        <div class="recent-grid">
          <div v-for="path in recentRepos" :key="path" class="recent-card" :title="path"
            @click="emit('selectRepo', path)">
            <div class="card-icon">
              <FolderGit2 :size="16" class="icon-primary" />
            </div>
            <div class="card-info">
              <span class="repo-name">{{ path.split('/').pop() || path }}</span>
              <span class="repo-path" :title="path">{{ path }}</span>
            </div>
            <button class="remove-btn" @click.stop="emit('removeRecent', path)" title="Remove from recent list">
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
        <BaseButton variant="secondary" size="xs" @click="emit('openSettings')">
          <Settings :size="12" />
          <span>Configure API Keys & Prompts</span>
        </BaseButton>
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/WelcomeScreen.css"></style>
