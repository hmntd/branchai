<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  RotateCcw,
  RotateCw,
  ArrowDownCircle,
  ArrowUpCircle,
  GitBranch,
  FolderArchive,
  Terminal,
  Settings,
  Sparkles,
  Search,
  Plus,
  RefreshCw,
  FolderMinus,
  FolderGit2,
  FolderPlus,
} from '@lucide/vue';
import ActionToast from '../molecules/ActionToast.vue';
import BaseModal from '../atoms/BaseModal.vue';
import BaseButton from '../atoms/BaseButton.vue';

import AccountBadge from '../molecules/AccountBadge.vue';
import { GitAccount } from '../../types/account';
import { User, Check, ChevronDown } from '@lucide/vue';

const props = defineProps<{
  repoPath: string;
  currentRepo: string;
  currentBranch: string;
  stagedCount: number;
  unstagedCount: number;
  lastCommitMsg?: string;
  activeProvider: string;
  activeModel: string;
  loading: boolean;
  recentRepos?: string[];
  accounts?: GitAccount[];
  activeAccount?: GitAccount | null;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
  (e: 'openSettings'): void;
  (e: 'toggleTerminal'): void;
  (e: 'closeRepo'): void;
  (e: 'browseRepo'): void;
  (e: 'selectRepo', path: string): void;
  (e: 'selectAccount', accountId: string): void;
}>();

const showAccountDropdown = ref(false);

const showBranchModal = ref(false);
const showRepoSearchModal = ref(false);
const repoSearchInput = ref('');
const newBranchName = ref('');
const isActionExecuting = ref(false);
const actionStatus = ref('');
const actionIsError = ref(false);
const undoneCommitMsg = ref('');

const filteredRecentRepos = computed(() => {
  const list = props.recentRepos || [];
  const q = repoSearchInput.value.toLowerCase().trim();
  if (!q) return list;
  return list.filter((r) => r.toLowerCase().includes(q));
});

function selectRepository(path: string) {
  if (!path || !path.trim()) return;
  emit('selectRepo', path.trim());
  showRepoSearchModal.value = false;
  repoSearchInput.value = '';
}

function handleEnterSearch() {
  if (repoSearchInput.value.trim()) {
    selectRepository(repoSearchInput.value.trim());
  } else if (filteredRecentRepos.value.length > 0) {
    selectRepository(filteredRecentRepos.value[0]);
  }
}

const undoTooltip = computed(() => {
  if (props.lastCommitMsg) {
    return `Undo last commit: "${props.lastCommitMsg}" (soft reset HEAD~1)`;
  }
  return 'Undo last commit (resets HEAD~1 soft)';
});

const redoTooltip = computed(() => {
  if (undoneCommitMsg.value) {
    return `Redo previously undone commit: "${undoneCommitMsg.value}"`;
  }
  return 'Redo previously undone commit';
});

const pullTooltip = computed(() => `Pull changes for branch '${props.currentBranch || 'master'}' from origin`);
const pushTooltip = computed(() => `Push commits on branch '${props.currentBranch || 'master'}' to origin`);
const branchTooltip = computed(() => `Create a new branch from '${props.currentBranch || 'master'}'`);
const stashTooltip = computed(() => `Stash ${props.stagedCount + props.unstagedCount} uncommitted file changes`);
const popTooltip = computed(() => 'Pop stashed changes back into working directory');

async function handleUndo() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('undo_commit', { repoPath: props.repoPath });
    actionStatus.value = res;
    undoneCommitMsg.value = props.lastCommitMsg || 'Undone commit';
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Undo failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handleRedo() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('redo_commit', { repoPath: props.repoPath });
    actionStatus.value = res;
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Redo failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handlePull() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('pull_changes', { repoPath: props.repoPath });
    actionStatus.value = res;
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Pull failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handlePush() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('push_changes', { repoPath: props.repoPath });
    actionStatus.value = res;
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Push failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handleStash() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('stash_save', { repoPath: props.repoPath });
    actionStatus.value = res;
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Stash failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handlePop() {
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('stash_pop', { repoPath: props.repoPath });
    actionStatus.value = res;
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Pop failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}

async function handleCreateBranch() {
  if (!newBranchName.value.trim()) return;
  isActionExecuting.value = true;
  actionStatus.value = '';
  actionIsError.value = false;
  try {
    const res: string = await invoke('create_new_branch', {
      repoPath: props.repoPath,
      branchName: newBranchName.value.trim(),
    });
    actionStatus.value = res;
    showBranchModal.value = false;
    newBranchName.value = '';
    emit('refresh');
  } catch (err: any) {
    actionStatus.value = typeof err === 'string' ? err : err.message || 'Branch creation failed';
    actionIsError.value = true;
  } finally {
    isActionExecuting.value = false;
  }
}
</script>

<template>
  <header class="topbar-container">
    <!-- Top Native Menu Bar -->
    <div class="native-menu-bar">
      <div class="menu-brand">
        <img src="/logo.png" alt="BranchAI Logo" class="menu-logo-img" />
        <span class="menu-app-name">BranchAI</span>
      </div>

      <div class="menu-spacer"></div>

      <!-- Active Git Account Indicator & Quick Switcher -->
      <div v-if="activeAccount" class="account-selector-wrap">
        <div class="account-pill-trigger" @click="showAccountDropdown = !showAccountDropdown">
          <AccountBadge :account="activeAccount" interactive />
          <ChevronDown :size="12" class="chevron-icon" />
        </div>

        <div v-if="showAccountDropdown" class="account-dropdown-menu" @click.self="showAccountDropdown = false">
          <div class="dropdown-title-row">ASSIGN GIT ACCOUNT FOR WORKSPACE</div>
          <div v-for="acc in (accounts || [])" :key="acc.id" class="dropdown-account-item"
            :class="{ active: acc.id === activeAccount?.id }"
            @click="emit('selectAccount', acc.id); showAccountDropdown = false;">
            <AccountBadge :account="acc" />
            <Check v-if="acc.id === activeAccount?.id" :size="14" class="text-success" />
          </div>
          <div class="dropdown-footer-action" @click="emit('openSettings'); showAccountDropdown = false;">
            <Settings :size="12" /> Manage Accounts in Settings...
          </div>
        </div>
      </div>

      <div v-else class="no-account-pill" @click="emit('openSettings')" title="Click to link a Git account">
        <User :size="12" />
        <span>Link Git Account</span>
      </div>

      <div class="ai-status-pill" @click="emit('openSettings')">
        <Sparkles :size="11" />
        <span>{{ activeProvider.toUpperCase() }} ({{ activeModel }})</span>
      </div>

      <button @click="emit('openSettings')" class="icon-btn" title="Settings">
        <Settings :size="13" />
      </button>
    </div>

    <!-- Action Toolbar with Instant Custom Tooltips (0ms delay) -->
    <div class="toolbar">
      <div class="repo-breadcrumb">
        <span class="label">repository</span>
        <span class="value">{{ currentRepo }}</span>
        <span class="sep">></span>
        <span class="label">branch</span>
        <span class="value-branch">{{ currentBranch || 'master' }}</span>

        <!-- Close Repository Button -->
        <div class="has-tooltip">
          <button class="close-repo-btn" @click="emit('closeRepo')">
            <FolderMinus :size="12" />
          </button>
          <div class="tooltip-box">Close active repository (return to Welcome Screen)</div>
        </div>
      </div>

      <div class="toolbar-actions">
        <!-- Undo Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handleUndo" :disabled="isActionExecuting">
            <RotateCcw :size="13" />
            <span>Undo</span>
          </button>
          <div class="tooltip-box">{{ undoTooltip }}</div>
        </div>

        <!-- Redo Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handleRedo" :disabled="isActionExecuting">
            <RotateCw :size="13" />
            <span>Redo</span>
          </button>
          <div class="tooltip-box">{{ redoTooltip }}</div>
        </div>

        <div class="v-divider"></div>

        <!-- Pull Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handlePull" :disabled="isActionExecuting">
            <ArrowDownCircle :size="13" />
            <span>Pull</span>
          </button>
          <div class="tooltip-box">{{ pullTooltip }}</div>
        </div>

        <!-- Push Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handlePush" :disabled="isActionExecuting">
            <ArrowUpCircle :size="13" />
            <span>Push</span>
          </button>
          <div class="tooltip-box">{{ pushTooltip }}</div>
        </div>

        <!-- Branch Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="showBranchModal = true">
            <GitBranch :size="13" />
            <span>Branch</span>
          </button>
          <div class="tooltip-box">{{ branchTooltip }}</div>
        </div>

        <!-- Stash Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handleStash" :disabled="isActionExecuting">
            <FolderArchive :size="13" />
            <span>Stash</span>
          </button>
          <div class="tooltip-box">{{ stashTooltip }}</div>
        </div>

        <!-- Pop Button -->
        <div class="has-tooltip">
          <button class="tool-btn" @click="handlePop" :disabled="isActionExecuting">
            <FolderArchive :size="13" />
            <span>Pop</span>
          </button>
          <div class="tooltip-box">{{ popTooltip }}</div>
        </div>

        <div class="v-divider"></div>

        <!-- Terminal Button -->
        <div class="has-tooltip">
          <button class="tool-btn active-term" @click="emit('toggleTerminal')">
            <Terminal :size="13" />
            <span>Terminal</span>
          </button>
          <div class="tooltip-box">Toggle embedded CLI terminal</div>
        </div>
      </div>

      <!-- Working Search Repository Button -->
      <div class="toolbar-search" @click="showRepoSearchModal = true" title="Find / Switch Repository">
        <Search :size="13" class="search-icon" />
        <span class="search-btn-label">Find repository...</span>
      </div>
    </div>

    <!-- Toast Action Banner Molecule -->
    <ActionToast :message="actionStatus" :is-error="actionIsError" />

    <!-- Create Branch Dialog Modal Atom -->
    <BaseModal v-if="showBranchModal" title="Create New Branch" @close="showBranchModal = false">
      <template #header-icon>
        <GitBranch :size="16" class="icon-primary" />
      </template>

      <p class="desc">Create a new branch from <strong>{{ currentBranch || 'master' }}</strong>:</p>
      <input v-model="newBranchName" @keyup.enter="handleCreateBranch" placeholder="e.g. feature/new-ai-model"
        class="branch-name-input" />

      <template #footer>
        <BaseButton variant="secondary" @click="showBranchModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" @click="handleCreateBranch"
          :disabled="!newBranchName.trim() || isActionExecuting">
          <RefreshCw v-if="isActionExecuting" :size="13" class="spinning" />
          <Plus v-else :size="13" />
          Create & Checkout Branch
        </BaseButton>
      </template>
    </BaseModal>

    <!-- Search / Switch Repository Dialog Modal Atom -->
    <BaseModal v-if="showRepoSearchModal" title="Find or Open Repository" size="wide"
      @close="showRepoSearchModal = false">
      <template #header-icon>
        <FolderGit2 :size="16" class="icon-primary" />
      </template>

      <div class="modal-search-box">
        <Search :size="14" class="search-box-icon" />
        <input v-model="repoSearchInput" @keyup.enter="handleEnterSearch"
          placeholder="Search recent or enter full repository path..." class="repo-path-input" autofocus />
      </div>

      <div class="recent-repos-section">
        <div class="section-title">Recent Repositories</div>
        <div v-if="filteredRecentRepos.length === 0" class="empty-recent">
          No matching repositories found. Press Enter to open path.
        </div>
        <div class="repo-results-list" v-else>
          <div v-for="repo in filteredRecentRepos" :key="repo" class="repo-result-item"
            :class="{ active: repo === repoPath }" :title="repo" @click="selectRepository(repo)">
            <FolderGit2 :size="14" class="repo-item-icon" />
            <div class="repo-item-info">
              <span class="repo-item-name">{{ repo.split('/').pop() || repo }}</span>
              <span class="repo-item-path" :title="repo">{{ repo }}</span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <BaseButton variant="secondary" @click="emit('browseRepo'); showRepoSearchModal = false">
          <FolderPlus :size="13" />
          Browse Folder...
        </BaseButton>
        <BaseButton variant="primary" @click="handleEnterSearch"
          :disabled="!repoSearchInput.trim() && filteredRecentRepos.length === 0">
          Open Repository
        </BaseButton>
      </template>
    </BaseModal>
  </header>
</template>

<style scoped src="../../styles/organisms/TopBar.css"></style>
