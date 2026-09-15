<script setup lang="ts">
import { ref, watch } from 'vue';
import {
  FolderGit2,
  GitBranch,
  ChevronDown,
  ChevronRight,
  ShieldCheck,
  AlertOctagon,
  Settings,
  Check,
  Globe,
  Search,
  Cloud,
  GitPullRequest,
  AlertCircle,
  Users,
  FolderPlus,
  FolderMinus,
} from '@lucide/vue';

const props = defineProps<{
  repoPath: string;
  branches: Array<{ name: string; is_head: boolean; is_remote: boolean }>;
  currentBranch: string;
  activeTab: string;
  recentRepos?: string[];
}>();

const emit = defineEmits<{
  (e: 'update:repoPath', path: string): void;
  (e: 'selectTab', tab: string): void;
  (e: 'openSettings'): void;
  (e: 'checkoutBranch', branchName: string): void;
  (e: 'browseRepo'): void;
  (e: 'closeRepo'): void;
}>();

const inputPath = ref(props.repoPath);
const filterText = ref('');

const isLocalOpen = ref(true);
const isRemoteOpen = ref(true);
const isCloudOpen = ref(false);
const isPrOpen = ref(false);
const isAiOpen = ref(true);

watch(() => props.repoPath, (newVal) => {
  inputPath.value = newVal;
});

function handleRepoSubmit() {
  if (inputPath.value.trim()) {
    emit('update:repoPath', inputPath.value.trim());
  }
}

function selectRepo(path: string) {
  inputPath.value = path;
  emit('update:repoPath', path);
}
</script>

<template>
  <aside class="left-sidebar gk-panel">
    <!-- Top Repository Selector Section -->
    <div class="repo-section">
      <div class="repo-header">
        <FolderGit2 :size="13" class="icon-primary" />
        <span class="title">REPOSITORY PATH</span>
        <button class="icon-action-btn danger" @click="emit('closeRepo')" title="Close active repository">
          <FolderMinus :size="12" />
        </button>
      </div>

      <div class="repo-input-box">
        <input
          v-model="inputPath"
          @keyup.enter="handleRepoSubmit"
          placeholder="Git repo path..."
          class="repo-input"
        />
        <button @click="emit('browseRepo')" class="btn btn-secondary btn-xs" title="Browse repository folder natively">
          <FolderPlus :size="11" /> Browse
        </button>
      </div>

      <!-- Quick Recent Repos Picker -->
      <div class="recent-repos" v-if="recentRepos && recentRepos.length > 0">
        <span class="recent-label">RECENT PROJECTS:</span>
        <div class="recent-tags">
          <span
            v-for="r in recentRepos.slice(0, 6)"
            :key="r"
            class="repo-tag"
            :class="{ active: repoPath === r }"
            @click="selectRepo(r)"
          >
            {{ r.split('/').pop() || r }}
          </span>
        </div>
      </div>
    </div>

    <hr class="divider" />

    <!-- Filter Input (Filter Ctrl + Alt + f) -->
    <div class="filter-box">
      <span class="viewing-count">Viewing 2</span>
      <div class="filter-input-wrap">
        <input
          v-model="filterText"
          placeholder="Filter (Ctrl + Alt + f)"
          class="filter-input"
        />
        <Search :size="11" class="search-icon" />
      </div>
    </div>

    <!-- Accordions Tree -->
    <div class="tree-container">
      <!-- LOCAL BRANCHES -->
      <div class="tree-group">
        <div class="group-header" @click="isLocalOpen = !isLocalOpen">
          <component :is="isLocalOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="group-title">LOCAL</span>
          <span class="count">{{ branches.filter(b => !b.is_remote).length || 1 }}</span>
        </div>
        <div v-if="isLocalOpen" class="group-body">
          <div
            v-for="b in branches.filter(b => !b.is_remote)"
            :key="b.name"
            class="tree-item"
            :class="{ active: b.is_head }"
            @dblclick="emit('checkoutBranch', b.name)"
            title="Double click to switch branch"
          >
            <Check v-if="b.is_head" :size="11" class="icon-success" />
            <GitBranch v-else :size="11" class="icon-muted" />
            <span class="item-name">{{ b.name }}</span>
          </div>
          <div v-if="branches.filter(b => !b.is_remote).length === 0" class="tree-item active">
            <Check :size="11" class="icon-success" />
            <span class="item-name">master</span>
          </div>
        </div>
      </div>

      <!-- REMOTE BRANCHES -->
      <div class="tree-group">
        <div class="group-header" @click="isRemoteOpen = !isRemoteOpen">
          <component :is="isRemoteOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="group-title">REMOTE</span>
          <span class="count">{{ branches.filter(b => b.is_remote).length || 1 }}</span>
        </div>
        <div v-if="isRemoteOpen" class="group-body">
          <div
            v-for="b in branches.filter(b => b.is_remote)"
            :key="b.name"
            class="tree-item"
            @dblclick="emit('checkoutBranch', b.name)"
            title="Double click to checkout branch"
          >
            <Globe :size="11" class="icon-muted" />
            <span class="item-name">{{ b.name }}</span>
          </div>
          <div v-if="branches.filter(b => b.is_remote).length === 0" class="tree-item">
            <Globe :size="11" class="icon-muted" />
            <span class="item-name">origin/master</span>
          </div>
        </div>
      </div>

      <!-- CLOUD PATCHES -->
      <div class="tree-group">
        <div class="group-header" @click="isCloudOpen = !isCloudOpen">
          <component :is="isCloudOpen ? ChevronDown : ChevronRight" :size="11" />
          <Cloud :size="11" class="icon-muted" />
          <span class="group-title">CLOUD PATCHES</span>
          <span class="count">0</span>
        </div>
      </div>

      <!-- PULL REQUESTS -->
      <div class="tree-group">
        <div class="group-header" @click="isPrOpen = !isPrOpen">
          <component :is="isPrOpen ? ChevronDown : ChevronRight" :size="11" />
          <GitPullRequest :size="11" class="icon-muted" />
          <span class="group-title">PULL REQUESTS</span>
          <span class="count">0</span>
        </div>
      </div>

      <!-- ISSUES & TEAMS -->
      <div class="tree-group">
        <div class="group-header">
          <ChevronRight :size="11" />
          <AlertCircle :size="11" class="icon-muted" />
          <span class="group-title">ISSUES</span>
        </div>
      </div>

      <div class="tree-group">
        <div class="group-header">
          <ChevronRight :size="11" />
          <Users :size="11" class="icon-muted" />
          <span class="group-title">TEAMS</span>
        </div>
      </div>

      <hr class="divider" />

      <!-- AI TOOLS & REVIEWS -->
      <div class="tree-group">
        <div class="group-header" @click="isAiOpen = !isAiOpen">
          <component :is="isAiOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="group-title text-purple">AI TOOLS & REVIEWS</span>
        </div>
        <div v-if="isAiOpen" class="group-body">
          <div
            class="tree-item"
            :class="{ active: activeTab === 'review' }"
            @click="emit('selectTab', 'review')"
          >
            <ShieldCheck :size="12" class="icon-purple" />
            <span class="item-name">AI Code Reviewer</span>
          </div>

          <div
            class="tree-item"
            :class="{ active: activeTab === 'conflicts' }"
            @click="emit('selectTab', 'conflicts')"
          >
            <AlertOctagon :size="12" class="icon-warning" />
            <span class="item-name">Conflict Assistant</span>
          </div>
        </div>
      </div>

      <div class="tree-item" @click="emit('openSettings')">
        <Settings :size="12" class="icon-muted" />
        <span class="item-name">Settings & API Keys</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.left-sidebar {
  width: 250px;
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-radius: 0;
  border-right: 1px solid var(--border-color);
  user-select: none;
  overflow-y: auto;
}

.repo-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
  background: rgba(0, 0, 0, 0.25);
}

.repo-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.repo-header .title {
  flex: 1;
}

.icon-action-btn {
  background: transparent;
  border: none;
  color: var(--text-dim);
  cursor: pointer;
  padding: 2px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  transition: all 0.15s ease;
}

.icon-action-btn.danger:hover {
  color: var(--danger);
  background: rgba(255, 71, 87, 0.2);
}

.repo-input-box {
  display: flex;
  gap: 6px;
}

.repo-input {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 6px;
}

.recent-repos {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 2px;
}

.recent-label {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-dim);
}

.recent-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.repo-tag {
  font-size: 10px;
  font-family: var(--font-mono);
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  padding: 2px 5px;
  border-radius: 3px;
  cursor: pointer;
  border: 1px solid transparent;
}

.repo-tag:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-main);
}

.repo-tag.active {
  background: rgba(0, 210, 211, 0.15);
  color: var(--primary);
  border-color: rgba(0, 210, 211, 0.3);
}

.divider {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 4px 0;
}

.filter-box {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 10px;
}

.viewing-count {
  font-size: 10px;
  color: var(--text-dim);
}

.filter-input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.filter-input {
  width: 100%;
  font-size: 11px;
  padding: 4px 24px 4px 8px;
  background: rgba(0, 0, 0, 0.3);
}

.filter-input-wrap .search-icon {
  position: absolute;
  right: 8px;
  color: var(--text-dim);
}

.tree-container {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 4px 6px;
}

.tree-group {
  display: flex;
  flex-direction: column;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 6px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  cursor: pointer;
}

.group-title {
  flex: 1;
  letter-spacing: 0.5px;
}

.count {
  font-size: 10px;
  color: var(--text-dim);
}

.group-body {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding-left: 12px;
}

.tree-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 3px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
}

.tree-item:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-main);
}

.tree-item.active {
  color: var(--primary);
  font-weight: 600;
  background: rgba(0, 210, 211, 0.12);
}

.text-purple { color: #9c88ff; }
.icon-purple { color: #9c88ff; }
.btn-xs { padding: 3px 6px; font-size: 10px; }
</style>
