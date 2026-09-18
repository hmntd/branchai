<script setup lang="ts">
import { ref, watch, computed } from 'vue';
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
  GitPullRequest,
  AlertCircle,
  FolderPlus,
  FolderMinus,
  Play,
} from '@lucide/vue';
import SearchBox from '../molecules/SearchBox.vue';
import RepoTag from '../molecules/RepoTag.vue';
import BaseButton from '../atoms/BaseButton.vue';

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
const isAiOpen = ref(true);

const prCount = computed(() => {
  return props.branches.filter((b) => !b.is_remote && !b.is_head && b.name !== 'main' && b.name !== 'master').length;
});

const issueCount = ref(0);

function loadIssueCount() {
  if (!props.repoPath || !props.repoPath.trim()) {
    issueCount.value = 0;
    return;
  }
  let count = 0;
  const storageKey = `branchai_issues_${props.repoPath.trim()}`;
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw) {
      const parsed: any[] = JSON.parse(raw);
      count += parsed.filter((i) => i.status === 'open').length;
    }
  } catch (e) { }
  issueCount.value = count;
}

watch(
  () => props.repoPath,
  (newVal) => {
    inputPath.value = newVal;
    loadIssueCount();
  },
  { immediate: true }
);

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
        <input v-model="inputPath" @keyup.enter="handleRepoSubmit" placeholder="Git repo path..." class="repo-input" />
        <BaseButton size="xs" variant="secondary" title="Browse repository folder natively" @click="emit('browseRepo')">
          <FolderPlus :size="11" /> Browse
        </BaseButton>
      </div>

      <!-- Quick Recent Repos Picker Molecule -->
      <div class="recent-repos" v-if="recentRepos && recentRepos.length > 0">
        <span class="recent-label">RECENT PROJECTS:</span>
        <div class="recent-tags">
          <RepoTag v-for="r in recentRepos.slice(0, 6)" :key="r" :path="r" :active="repoPath === r"
            @select="selectRepo" />
        </div>
      </div>
    </div>

    <hr class="divider" />

    <!-- Filter Input Molecule -->
    <div class="filter-box">
      <span class="viewing-count">Viewing 2</span>
      <SearchBox v-model="filterText" placeholder="Filter (Ctrl + Alt + f)" />
    </div>

    <!-- Accordions Tree -->
    <div class="tree-container">
      <!-- LOCAL BRANCHES -->
      <div class="tree-group">
        <div class="group-header" @click="isLocalOpen = !isLocalOpen">
          <component :is="isLocalOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="group-title">LOCAL</span>
          <span class="count">{{branches.filter(b => !b.is_remote).length || 1}}</span>
        </div>
        <div v-if="isLocalOpen" class="group-body">
          <div v-for="b in branches.filter(b => !b.is_remote)" :key="b.name" class="tree-item"
            :class="{ active: b.is_head }" @dblclick="emit('checkoutBranch', b.name)"
            title="Double click to switch branch">
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
          <span class="count">{{branches.filter(b => b.is_remote).length || 1}}</span>
        </div>
        <div v-if="isRemoteOpen" class="group-body">
          <div v-for="b in branches.filter(b => b.is_remote)" :key="b.name" class="tree-item"
            @dblclick="emit('checkoutBranch', b.name)" title="Double click to checkout branch">
            <Globe :size="11" class="icon-muted" />
            <span class="item-name">{{ b.name }}</span>
          </div>
          <div v-if="branches.filter(b => b.is_remote).length === 0" class="tree-item">
            <Globe :size="11" class="icon-muted" />
            <span class="item-name">origin/master</span>
          </div>
        </div>
      </div>

      <!-- PULL REQUESTS -->
      <div class="tree-group">
        <div class="group-header" :class="{ active: activeTab === 'pull_requests' }"
          @click="emit('selectTab', 'pull_requests')">
          <GitPullRequest :size="12" :class="activeTab === 'pull_requests' ? 'icon-primary' : 'icon-muted'" />
          <span class="group-title">PULL REQUESTS</span>
          <span class="count-pill">{{ prCount }}</span>
        </div>
      </div>

      <!-- ISSUES -->
      <div class="tree-group">
        <div class="group-header" :class="{ active: activeTab === 'issues' }" @click="emit('selectTab', 'issues')">
          <AlertCircle :size="12" :class="activeTab === 'issues' ? 'icon-warning' : 'icon-muted'" />
          <span class="group-title">ISSUES</span>
          <span class="count-pill warning">{{ issueCount }}</span>
        </div>
      </div>

      <!-- GITHUB ACTIONS -->
      <div class="tree-group">
        <div class="group-header" :class="{ active: activeTab === 'actions' }" @click="emit('selectTab', 'actions')">
          <Play :size="12" :class="activeTab === 'actions' ? 'icon-purple' : 'icon-muted'" />
          <span class="group-title">GITHUB ACTIONS</span>
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
          <div class="tree-item" :class="{ active: activeTab === 'review' }" @click="emit('selectTab', 'review')">
            <ShieldCheck :size="12" class="icon-purple" />
            <span class="item-name">AI Code Reviewer</span>
          </div>

          <div class="tree-item" :class="{ active: activeTab === 'conflicts' }" @click="emit('selectTab', 'conflicts')">
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

<style scoped src="../../styles/organisms/LeftSidebar.css"></style>
