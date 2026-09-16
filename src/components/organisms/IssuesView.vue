<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  AlertCircle,
  CheckCircle2,
  Plus,
  MessageSquare,
  ArrowLeft,
  Send,
  Tag,
  Filter,
} from '@lucide/vue';
import SearchBox from '../molecules/SearchBox.vue';
import BaseBadge from '../atoms/BaseBadge.vue';
import BaseButton from '../atoms/BaseButton.vue';
import BaseModal from '../atoms/BaseModal.vue';

export interface IssueComment {
  id: string;
  author: string;
  text: string;
  timestamp: string;
}

export interface IssueLabel {
  name: string;
  color: string;
}

export interface Issue {
  id: number;
  title: string;
  description: string;
  author: string;
  status: 'open' | 'closed';
  priority: 'low' | 'medium' | 'high' | 'critical';
  labels: IssueLabel[];
  createdAt: string;
  updatedAt: string;
  comments: IssueComment[];
}

const props = defineProps<{
  repoPath: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

// Real repository issues array (no sample data)
const issues = ref<Issue[]>([]);
const loadingIssues = ref(false);

async function saveLocalIssues() {
  if (!props.repoPath || !props.repoPath.trim()) return;
  try {
    await invoke('save_repo_issues', {
      repoPath: props.repoPath,
      issuesJson: JSON.stringify(issues.value),
    });
  } catch (e) {
    console.error('Failed to save repo issues to .git storage:', e);
  }
}

async function loadRepoIssues() {
  if (!props.repoPath || !props.repoPath.trim()) {
    issues.value = [];
    return;
  }

  loadingIssues.value = true;
  selectedIssue.value = null;
  const loadedList: Issue[] = [];

  // 1. Load saved repository issues from .git/branchai_issues.json
  try {
    const rawIssues: string = await invoke('get_repo_issues', { repoPath: props.repoPath });
    if (rawIssues && rawIssues.trim()) {
      const parsed: Issue[] = JSON.parse(rawIssues);
      loadedList.push(...parsed);
    }
  } catch (e) {
    console.error('Failed to parse repo stored issues:', e);
  }

  // 2. Fetch real GitHub Issues if remote origin is a GitHub repository
  try {
    const remoteInfo: { owner?: string; repo_name?: string } = await invoke('get_repo_remote_info', {
      repoPath: props.repoPath,
    });

    if (remoteInfo.owner && remoteInfo.repo_name) {
      try {
        const res = await fetch(`https://api.github.com/repos/${remoteInfo.owner}/${remoteInfo.repo_name}/issues?state=all&per_page=30`);
        if (res.ok) {
          const ghIssues: any[] = await res.json();
          for (const gh of ghIssues) {
            // Ignore PR entries returned by GitHub issues API
            if (!gh.pull_request && !loadedList.some((i) => i.id === gh.number)) {
              loadedList.push({
                id: gh.number,
                title: gh.title,
                description: gh.body || 'No description provided.',
                author: gh.user?.login || 'GitHub User',
                status: gh.state === 'closed' ? 'closed' : 'open',
                priority: 'medium',
                labels: (gh.labels || []).map((l: any) => ({
                  name: l.name,
                  color: `#${l.color}`,
                })),
                createdAt: new Date(gh.created_at).toLocaleString(),
                updatedAt: new Date(gh.updated_at).toLocaleString(),
                comments: [],
              });
            }
          }
        }
      } catch (ghErr) {
        console.log('GitHub Issues API fetch skipped or offline:', ghErr);
      }
    }
  } catch (err) {
    console.error('Error fetching repo remote info:', err);
  } finally {
    issues.value = loadedList;
    loadingIssues.value = false;
  }
}

watch(
  () => props.repoPath,
  () => {
    loadRepoIssues();
  },
  { immediate: true }
);

const statusFilter = ref<'all' | 'open' | 'closed'>('open');
const labelFilter = ref<string>('all');
const searchQuery = ref('');
const selectedIssue = ref<Issue | null>(null);

// New Issue Modal state
const showNewIssueModal = ref(false);
const newIssueTitle = ref('');
const newIssueDescription = ref('');
const newIssuePriority = ref<'low' | 'medium' | 'high' | 'critical'>('medium');
const selectedLabelNames = ref<string[]>(['bug']);

const newCommentText = ref('');

const allAvailableLabels: IssueLabel[] = [
  { name: 'bug', color: '#ff4757' },
  { name: 'feature', color: '#2ed573' },
  { name: 'performance', color: '#ffa502' },
  { name: 'ui/ux', color: '#00d2d3' },
  { name: 'enhancement', color: '#9c88ff' },
  { name: 'documentation', color: '#70a1ff' },
];

const filteredIssues = computed(() => {
  return issues.value.filter((issue) => {
    const matchesStatus = statusFilter.value === 'all' || issue.status === statusFilter.value;
    const matchesLabel =
      labelFilter.value === 'all' || issue.labels.some((l) => l.name === labelFilter.value);

    const q = searchQuery.value.toLowerCase().trim();
    const matchesQuery =
      !q ||
      issue.title.toLowerCase().includes(q) ||
      issue.description.toLowerCase().includes(q) ||
      issue.author.toLowerCase().includes(q) ||
      `#${issue.id}`.includes(q);

    return matchesStatus && matchesLabel && matchesQuery;
  });
});

const openCount = computed(() => issues.value.filter((i) => i.status === 'open').length);
const closedCount = computed(() => issues.value.filter((i) => i.status === 'closed').length);

function toggleLabelSelection(labelName: string) {
  const idx = selectedLabelNames.value.indexOf(labelName);
  if (idx !== -1) {
    selectedLabelNames.value.splice(idx, 1);
  } else {
    selectedLabelNames.value.push(labelName);
  }
}

async function createIssue() {
  if (!newIssueTitle.value.trim()) return;

  const chosenLabels = allAvailableLabels.filter((l) => selectedLabelNames.value.includes(l.name));

  const newIssue: Issue = {
    id: issues.value.length + 1,
    title: newIssueTitle.value.trim(),
    description: newIssueDescription.value.trim() || 'No description provided.',
    author: 'You',
    status: 'open',
    priority: newIssuePriority.value,
    labels: chosenLabels.length > 0 ? chosenLabels : [{ name: 'enhancement', color: '#9c88ff' }],
    createdAt: new Date().toLocaleString(),
    updatedAt: new Date().toLocaleString(),
    comments: [],
  };

  issues.value.unshift(newIssue);
  selectedIssue.value = newIssue;
  showNewIssueModal.value = false;
  newIssueTitle.value = '';
  newIssueDescription.value = '';
  await saveLocalIssues();
}

async function submitComment() {
  if (!newCommentText.value.trim() || !selectedIssue.value) return;

  selectedIssue.value.comments.push({
    id: `c_${Date.now()}`,
    author: 'You',
    text: newCommentText.value.trim(),
    timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
  });
  newCommentText.value = '';
  await saveLocalIssues();
}

async function toggleIssueStatus() {
  if (!selectedIssue.value) return;
  if (selectedIssue.value.status === 'open') {
    selectedIssue.value.status = 'closed';
  } else {
    selectedIssue.value.status = 'open';
  }
  selectedIssue.value.updatedAt = new Date().toLocaleString();
  await saveLocalIssues();
}

function getPriorityBadgeClass(priority: string) {
  if (priority === 'critical' || priority === 'high') return 'badge-danger';
  if (priority === 'medium') return 'badge-warning';
  return 'badge-muted';
}
</script>

<template>
  <div class="issues-workspace gk-panel">
    <!-- Header -->
    <div class="issues-header">
      <div class="header-title-box">
        <AlertCircle :size="18" class="icon-warning" />
        <span class="title">ISSUES TRACKER</span>
        <BaseBadge variant="branch">{{ openCount }} Open</BaseBadge>
      </div>

      <div class="header-actions">
        <BaseButton variant="primary" size="xs" @click="showNewIssueModal = true">
          <Plus :size="12" /> New Issue
        </BaseButton>
        <BaseButton variant="secondary" size="xs" @click="emit('close')">
          <ArrowLeft :size="12" /> Close View
        </BaseButton>
      </div>
    </div>

    <div class="issues-body">
      <!-- LIST VIEW -->
      <template v-if="!selectedIssue">
        <div class="issues-filter-bar">
          <div class="status-tabs">
            <button class="status-tab" :class="{ active: statusFilter === 'open' }" @click="statusFilter = 'open'">
              Open <span class="tab-count">{{ openCount }}</span>
            </button>
            <button class="status-tab" :class="{ active: statusFilter === 'closed' }" @click="statusFilter = 'closed'">
              Closed <span class="tab-count">{{ closedCount }}</span>
            </button>
            <button class="status-tab" :class="{ active: statusFilter === 'all' }" @click="statusFilter = 'all'">
              All <span class="tab-count">{{ issues.length }}</span>
            </button>
          </div>

          <div class="filter-controls">
            <!-- Label Dropdown Filter -->
            <div class="label-filter-wrap">
              <Filter :size="11" class="icon-muted" />
              <select v-model="labelFilter" class="label-select">
                <option value="all">All Labels</option>
                <option v-for="l in allAvailableLabels" :key="l.name" :value="l.name">
                  {{ l.name }}
                </option>
              </select>
            </div>

            <SearchBox v-model="searchQuery" placeholder="Search issues..." />
          </div>
        </div>

        <div class="issues-list">
          <div v-for="issue in filteredIssues" :key="issue.id" class="issue-card gk-card"
            @click="selectedIssue = issue">
            <div class="issue-card-left">
              <AlertCircle v-if="issue.status === 'open'" :size="16" class="icon-warning" />
              <CheckCircle2 v-else :size="16" class="icon-purple" />

              <div class="issue-info">
                <div class="issue-title-row">
                  <span class="issue-title">{{ issue.title }}</span>
                  <span class="issue-id">#{{ issue.id }}</span>

                  <span class="badge priority-badge" :class="getPriorityBadgeClass(issue.priority)">
                    {{ issue.priority.toUpperCase() }}
                  </span>
                </div>

                <div class="issue-meta-row">
                  <div class="labels-list">
                    <span v-for="lbl in issue.labels" :key="lbl.name" class="label-tag"
                      :style="{ backgroundColor: lbl.color + '22', color: lbl.color, borderColor: lbl.color + '55' }">
                      <Tag :size="9" /> {{ lbl.name }}
                    </span>
                  </div>

                  <span class="dot">•</span>
                  <span>Opened by <strong>{{ issue.author }}</strong></span>
                  <span class="dot">•</span>
                  <span>{{ issue.createdAt }}</span>
                </div>
              </div>
            </div>

            <div class="issue-card-right">
              <div class="comment-count">
                <MessageSquare :size="12" />
                <span>{{ issue.comments.length }}</span>
              </div>
            </div>
          </div>

          <div v-if="filteredIssues.length === 0" class="empty-state">
            <AlertCircle :size="32" class="icon-muted" />
            <p>No issues found matching the selected filters.</p>
          </div>
        </div>
      </template>

      <!-- DETAIL VIEW -->
      <template v-else>
        <div class="issue-detail-layout">
          <div class="detail-header">
            <BaseButton variant="secondary" size="xs" class="back-btn" @click="selectedIssue = null">
              <ArrowLeft :size="12" /> Back to Issues
            </BaseButton>

            <div class="detail-title-row">
              <h2 class="detail-title">{{ selectedIssue.title }}</h2>
              <span class="detail-id">#{{ selectedIssue.id }}</span>
              <BaseBadge :variant="selectedIssue.status === 'open' ? 'modified' : 'purple'">
                {{ selectedIssue.status.toUpperCase() }}
              </BaseBadge>
              <span class="badge" :class="getPriorityBadgeClass(selectedIssue.priority)">
                {{ selectedIssue.priority.toUpperCase() }}
              </span>
            </div>

            <div class="detail-meta-bar">
              <span>Opened by <strong>{{ selectedIssue.author }}</strong> • {{ selectedIssue.createdAt }}</span>
              <span class="dot">•</span>
              <div class="labels-list">
                <span v-for="lbl in selectedIssue.labels" :key="lbl.name" class="label-tag"
                  :style="{ backgroundColor: lbl.color + '22', color: lbl.color, borderColor: lbl.color + '55' }">
                  <Tag :size="9" /> {{ lbl.name }}
                </span>
              </div>
            </div>

            <div class="detail-actions">
              <BaseButton variant="secondary" size="xs" @click="toggleIssueStatus">
                <CheckCircle2 v-if="selectedIssue.status === 'open'" :size="12" class="icon-success" />
                <AlertCircle v-else :size="12" class="icon-warning" />
                {{ selectedIssue.status === 'open' ? 'Close Issue' : 'Reopen Issue' }}
              </BaseButton>
            </div>
          </div>

          <div class="detail-body">
            <!-- Issue Description -->
            <div class="description-card gk-card">
              <div class="card-header-sm">ISSUE DESCRIPTION</div>
              <div class="card-text">{{ selectedIssue.description }}</div>
            </div>

            <!-- Comments Timeline -->
            <div class="comments-timeline">
              <div class="section-title">COMMENTS ({{ selectedIssue.comments.length }})</div>
              <div v-for="comment in selectedIssue.comments" :key="comment.id" class="comment-card gk-card">
                <div class="comment-header">
                  <span class="comment-author">{{ comment.author }}</span>
                  <span class="comment-time">{{ comment.timestamp }}</span>
                </div>
                <div class="comment-text">{{ comment.text }}</div>
              </div>

              <div v-if="selectedIssue.comments.length === 0" class="empty-comments">
                No comments posted yet.
              </div>
            </div>

            <!-- Add Comment Box -->
            <div class="add-comment-box gk-card">
              <textarea v-model="newCommentText" placeholder="Leave a comment..." rows="3"
                class="comment-textarea"></textarea>
              <div class="comment-actions">
                <BaseButton variant="primary" size="xs" @click="submitComment" :disabled="!newCommentText.trim()">
                  <Send :size="11" /> Post Comment
                </BaseButton>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Create New Issue Modal Atom -->
    <BaseModal v-if="showNewIssueModal" title="Create New Issue" @close="showNewIssueModal = false">
      <template #header-icon>
        <AlertCircle :size="15" class="icon-warning" />
      </template>

      <div class="form-group">
        <label>Title</label>
        <input v-model="newIssueTitle" placeholder="Issue title..." class="form-input" />
      </div>

      <div class="form-group">
        <label>Priority</label>
        <select v-model="newIssuePriority" class="form-select">
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>

      <div class="form-group">
        <label>Labels</label>
        <div class="label-picker">
          <span v-for="lbl in allAvailableLabels" :key="lbl.name" class="label-picker-tag"
            :class="{ selected: selectedLabelNames.includes(lbl.name) }" :style="{
              backgroundColor: selectedLabelNames.includes(lbl.name) ? lbl.color + '33' : 'transparent',
              color: selectedLabelNames.includes(lbl.name) ? lbl.color : 'var(--text-muted)',
              borderColor: selectedLabelNames.includes(lbl.name) ? lbl.color : 'var(--border-color)',
            }" @click="toggleLabelSelection(lbl.name)">
            {{ lbl.name }}
          </span>
        </div>
      </div>

      <div class="form-group">
        <label>Description</label>
        <textarea v-model="newIssueDescription" placeholder="Describe the issue..." rows="4"
          class="form-textarea"></textarea>
      </div>

      <template #footer>
        <BaseButton variant="secondary" size="xs" @click="showNewIssueModal = false">Cancel</BaseButton>
        <BaseButton variant="primary" size="xs" @click="createIssue" :disabled="!newIssueTitle.trim()">
          Create Issue
        </BaseButton>
      </template>
    </BaseModal>
  </div>
</template>

<style scoped src="../../styles/organisms/IssuesView.css"></style>
