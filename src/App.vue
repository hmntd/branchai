<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

import TopBar from './components/TopBar.vue';
import LeftSidebar from './components/LeftSidebar.vue';
import BranchGraph, { GraphNode } from './components/BranchGraph.vue';
import RightStagingPanel, { FileStatus } from './components/RightStagingPanel.vue';
import AiCodeReview from './components/AiCodeReview.vue';
import ConflictResolver from './components/ConflictResolver.vue';
import SettingsModal, { AppConfig } from './components/SettingsModal.vue';
import TerminalDrawer from './components/TerminalDrawer.vue';
import WelcomeScreen from './components/WelcomeScreen.vue';

type Tab = 'graph' | 'review' | 'conflicts';

const activeTab = ref<Tab>('graph');
const repoPath = ref<string>('');
const loading = ref<boolean>(false);
const showSettings = ref<boolean>(false);
const showTerminal = ref<boolean>(false);

const nodes = ref<GraphNode[]>([]);
const files = ref<FileStatus[]>([]);
const branches = ref<Array<{ name: string; is_head: boolean; is_remote: boolean }>>([]);
const currentBranch = ref<string>('');

const config = ref<AppConfig>({
  openai_key: '',
  claude_key: '',
  gemini_key: '',
  grok_key: '',
  active_provider: 'openai',
  active_model: 'gpt-4o',
  custom_prompt_commit: '',
  custom_prompt_review: '',
  custom_prompt_conflict: '',
  last_opened_repo: null,
  recent_repos: [],
});

const isZeroState = computed(() => !repoPath.value.trim());

const stagedCount = computed(() => files.value.filter((f) => f.staged).length);
const unstagedCount = computed(() => files.value.filter((f) => !f.staged).length);
const modifiedCount = computed(() => files.value.filter((f) => f.status.includes('modified')).length);
const addedCount = computed(() => files.value.filter((f) => f.status.includes('new') || f.status === 'untracked').length);
const lastCommitMsg = computed(() => (nodes.value.length > 0 ? nodes.value[0].message : ''));

async function loadConfig() {
  try {
    const cfg: AppConfig = await invoke('get_config');
    config.value = cfg;

    if (cfg.last_opened_repo && cfg.last_opened_repo.trim()) {
      repoPath.value = cfg.last_opened_repo.trim();
      await refreshRepo();
    }
  } catch (e) {
    console.error('Failed to load config:', e);
  }
}

async function refreshRepo() {
  if (!repoPath.value.trim()) return;
  loading.value = true;
  try {
    const graphData: GraphNode[] = await invoke('get_graph_data', { repoPath: repoPath.value });
    nodes.value = graphData;

    const status: { current_branch: string; files: FileStatus[] } = await invoke('get_repo_status', {
      repoPath: repoPath.value,
    });
    files.value = status.files;
    currentBranch.value = status.current_branch;

    const branchList: Array<{ name: string; is_head: boolean; is_remote: boolean }> = await invoke('get_branches', {
      repoPath: repoPath.value,
    });
    branches.value = branchList;
  } catch (err) {
    console.error('Error reading repository:', err);
  } finally {
    loading.value = false;
  }
}

async function openRepo(path: string) {
  if (!path || !path.trim()) return;
  const cleanPath = path.trim();
  repoPath.value = cleanPath;

  const recent = [...(config.value.recent_repos || [])];
  const idx = recent.indexOf(cleanPath);
  if (idx !== -1) {
    recent.splice(idx, 1);
  }
  recent.unshift(cleanPath);
  config.value.recent_repos = recent;
  config.value.last_opened_repo = cleanPath;

  try {
    await invoke('save_config', { config: config.value });
  } catch (e) {
    console.error('Failed to save config:', e);
  }

  await refreshRepo();
}

async function handleBrowseRepo() {
  try {
    const selected: string | null = await invoke('pick_repository_folder');
    if (selected) {
      await openRepo(selected);
    }
  } catch (err) {
    console.error('Failed to pick folder:', err);
  }
}

async function handleCloseRepo() {
  repoPath.value = '';
  config.value.last_opened_repo = null;
  try {
    await invoke('save_config', { config: config.value });
  } catch (e) {
    console.error('Failed to save config:', e);
  }
}

async function handleRemoveRecent(path: string) {
  config.value.recent_repos = (config.value.recent_repos || []).filter((r) => r !== path);
  if (config.value.last_opened_repo === path) {
    config.value.last_opened_repo = null;
  }
  try {
    await invoke('save_config', { config: config.value });
  } catch (e) {
    console.error('Failed to save config:', e);
  }
}

function handleRepoPathUpdate(newPath: string) {
  openRepo(newPath);
}

function handleSelectWip() {
  activeTab.value = 'graph';
}

async function handleCheckoutBranch(branchName: string) {
  if (!branchName || branchName === currentBranch.value) return;
  try {
    loading.value = true;
    await invoke('checkout_branch', { repoPath: repoPath.value, branchName });
    await refreshRepo();
  } catch (err: any) {
    console.error('Failed to checkout branch:', err);
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await loadConfig();
});
</script>

<template>
  <div class="app-layout">
    <!-- Zero-State Welcome Screen when no repository is open -->
    <WelcomeScreen
      v-if="isZeroState"
      :recent-repos="config.recent_repos || []"
      :active-provider="config.active_provider"
      :active-model="config.active_model"
      @browse="handleBrowseRepo"
      @select-repo="openRepo"
      @remove-recent="handleRemoveRecent"
      @open-settings="showSettings = true"
    />

    <!-- Active Repository Main Workspace -->
    <template v-else>
      <!-- Top Action Toolbar -->
      <TopBar
        :repo-path="repoPath"
        :current-repo="repoPath.split('/').pop() || 'BranchAI'"
        :current-branch="currentBranch"
        :staged-count="stagedCount"
        :unstaged-count="unstagedCount"
        :last-commit-msg="lastCommitMsg"
        :active-provider="config.active_provider"
        :active-model="config.active_model"
        :loading="loading"
        @refresh="refreshRepo"
        @open-settings="showSettings = true"
        @toggle-terminal="showTerminal = !showTerminal"
        @close-repo="handleCloseRepo"
        @browse-repo="handleBrowseRepo"
      />

      <!-- GitKraken 3-Pane Main Layout -->
      <div class="main-content">
        <!-- 1. Left Sidebar -->
        <LeftSidebar
          :repo-path="repoPath"
          :branches="branches"
          :current-branch="currentBranch"
          :active-tab="activeTab"
          :recent-repos="config.recent_repos || []"
          @update:repo-path="handleRepoPathUpdate"
          @select-tab="(t) => (activeTab = t as Tab)"
          @open-settings="showSettings = true"
          @checkout-branch="handleCheckoutBranch"
          @close-repo="handleCloseRepo"
          @browse-repo="handleBrowseRepo"
        />

        <!-- 2. Middle Pane -->
        <main class="center-workspace">
          <BranchGraph
            v-if="activeTab === 'graph'"
            :nodes="nodes"
            :current-branch="currentBranch"
            :has-changes="files.length > 0"
            :modified-count="modifiedCount"
            :added-count="addedCount"
            @select-wip="handleSelectWip"
            @checkout-branch="handleCheckoutBranch"
          />
          <AiCodeReview
            v-else-if="activeTab === 'review'"
            :repo-path="repoPath"
            @close="activeTab = 'graph'"
          />
          <ConflictResolver
            v-else-if="activeTab === 'conflicts'"
            :repo-path="repoPath"
            @resolved="refreshRepo"
            @close="activeTab = 'graph'"
          />

          <!-- Embedded Terminal Drawer -->
          <TerminalDrawer
            v-if="showTerminal"
            :repo-path="repoPath"
            @close="showTerminal = false"
          />
        </main>

        <!-- 3. Right Sidebar -->
        <RightStagingPanel
          :repo-path="repoPath"
          :files="files"
          :current-branch="currentBranch"
          @refresh="refreshRepo"
        />
      </div>
    </template>

    <!-- Settings Modal -->
    <SettingsModal
      v-if="showSettings"
      @close="showSettings = false"
      @saved="loadConfig"
    />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background-color: var(--bg-dark);
  color: var(--text-main);
  overflow: hidden;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.center-workspace {
  flex: 1;
  height: 100%;
  overflow: hidden;
  position: relative;
  background: var(--bg-dark);
}
</style>