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
  X,
  Check,
  AlertCircle,
  RefreshCw,
  FolderMinus,
} from '@lucide/vue';

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
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
  (e: 'openSettings'): void;
  (e: 'toggleTerminal'): void;
  (e: 'closeRepo'): void;
  (e: 'browseRepo'): void;
}>();

const showBranchModal = ref(false);
const newBranchName = ref('');
const isActionExecuting = ref(false);
const actionStatus = ref('');
const actionIsError = ref(false);
const undoneCommitMsg = ref('');

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
      <span class="menu-item">File</span>
      <span class="menu-item">Edit</span>
      <span class="menu-item">View</span>
      <span class="menu-item">Help</span>

      <div class="menu-spacer"></div>

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

      <div class="toolbar-search">
        <Search :size="13" class="search-icon" />
      </div>
    </div>

    <!-- Toast Action Banner -->
    <div v-if="actionStatus" class="action-toast" :class="actionIsError ? 'toast-error' : 'toast-success'">
      <AlertCircle v-if="actionIsError" :size="12" />
      <Check v-else :size="12" />
      <span>{{ actionStatus }}</span>
    </div>

    <!-- Create Branch Dialog Modal -->
    <div v-if="showBranchModal" class="modal-overlay" @click.self="showBranchModal = false">
      <div class="modal-card gk-panel">
        <div class="modal-header">
          <GitBranch :size="16" class="icon-primary" />
          <span class="title">Create New Branch</span>
          <button class="icon-btn" @click="showBranchModal = false"><X :size="16" /></button>
        </div>
        <div class="modal-body">
          <p class="desc">Create a new branch from <strong>{{ currentBranch || 'master' }}</strong>:</p>
          <input
            v-model="newBranchName"
            @keyup.enter="handleCreateBranch"
            placeholder="e.g. feature/new-ai-model"
            class="branch-name-input"
          />
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showBranchModal = false">Cancel</button>
          <button
            class="btn btn-primary"
            @click="handleCreateBranch"
            :disabled="!newBranchName.trim() || isActionExecuting"
          >
            <RefreshCw v-if="isActionExecuting" :size="13" class="spinning" />
            <Plus v-else :size="13" />
            Create & Checkout Branch
          </button>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar-container {
  display: flex;
  flex-direction: column;
  background: #181a1f;
  border-bottom: 1px solid var(--border-color);
  user-select: none;
  position: relative;
}

.native-menu-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  height: 26px;
  padding: 0 10px;
  background: #121417;
  font-size: 11px;
  color: var(--text-muted);
}

.menu-brand {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-right: 6px;
}

.menu-logo-img {
  width: 16px;
  height: 16px;
  border-radius: 4px;
}

.menu-app-name {
  font-weight: 700;
  color: var(--text-main);
  font-size: 11px;
  letter-spacing: 0.3px;
}

.menu-item {
  cursor: pointer;
}
.menu-item:hover {
  color: var(--text-main);
}

.menu-spacer {
  flex: 1;
}

.ai-status-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  font-weight: 600;
  color: #9c88ff;
  background: rgba(156, 136, 255, 0.15);
  border: 1px solid rgba(156, 136, 255, 0.3);
  padding: 2px 7px;
  border-radius: 10px;
  cursor: pointer;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 38px;
  padding: 0 12px;
  background: #1c2026;
  border-bottom: 1px solid var(--border-color);
}

.repo-breadcrumb {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
}

.repo-breadcrumb .label {
  color: var(--text-dim);
  font-size: 10px;
}

.repo-breadcrumb .value {
  font-weight: 600;
  color: var(--text-main);
}

.repo-breadcrumb .value-branch {
  font-weight: 600;
  color: var(--primary);
}

.repo-breadcrumb .sep {
  color: var(--text-dim);
}

.close-repo-btn {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  margin-left: 4px;
  transition: all 0.15s ease;
}

.close-repo-btn:hover {
  background: rgba(255, 71, 87, 0.2);
  color: var(--danger);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 3px;
}

.has-tooltip {
  position: relative;
  display: inline-flex;
}

.has-tooltip .tooltip-box {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 6px;
  background: #121417;
  color: var(--text-main);
  border: 1px solid var(--primary);
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 500;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  visibility: hidden;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.7);
  z-index: 1000;
}

.has-tooltip:hover .tooltip-box {
  opacity: 1;
  visibility: visible;
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  padding: 4px 8px;
  border-radius: 3px;
  cursor: pointer;
  transition: background 0.15s;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-main);
}

.active-term {
  color: var(--primary);
}
.active-term:hover {
  background: rgba(0, 210, 211, 0.15);
}

.v-divider {
  width: 1px;
  height: 14px;
  background: rgba(255, 255, 255, 0.1);
  margin: 0 3px;
}

.toolbar-search {
  display: flex;
  align-items: center;
}

.search-icon {
  color: var(--text-dim);
  cursor: pointer;
}
.search-icon:hover {
  color: var(--text-main);
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 3px;
}
.icon-btn:hover {
  color: var(--text-main);
}

.action-toast {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 500;
}
.toast-success {
  background: rgba(46, 213, 115, 0.2);
  color: var(--success);
}
.toast-error {
  background: rgba(255, 71, 87, 0.2);
  color: var(--danger);
}

/* Modal Overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-card {
  width: 400px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 13px;
}

.modal-header .title {
  flex: 1;
}

.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.desc {
  font-size: 12px;
  color: var(--text-muted);
}

.branch-name-input {
  font-family: var(--font-mono);
  font-size: 12px;
  width: 100%;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.spinning { animation: spin 1s linear infinite; }
@keyframes spin { 100% { transform: rotate(360deg); } }
</style>
