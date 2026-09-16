<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  AlertCircle,
  CheckCircle2,
  Plus,
  Search,
  MessageSquare,
  ArrowLeft,
  Send,
  Tag,
  Filter,
} from '@lucide/vue';

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

function saveLocalIssues() {
  if (!props.repoPath) return;
  const storageKey = `branchai_issues_${props.repoPath.trim()}`;
  localStorage.setItem(storageKey, JSON.stringify(issues.value));
}

async function loadRepoIssues() {
  if (!props.repoPath || !props.repoPath.trim()) {
    issues.value = [];
    return;
  }

  loadingIssues.value = true;
  selectedIssue.value = null;
  const loadedList: Issue[] = [];

  // 1. Load saved issues for this specific repo path from localStorage
  const storageKey = `branchai_issues_${props.repoPath.trim()}`;
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw) {
      const parsed: Issue[] = JSON.parse(raw);
      loadedList.push(...parsed);
    }
  } catch (e) {
    console.error('Failed to parse local stored issues:', e);
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

function createIssue() {
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
    createdAt: 'Just now',
    updatedAt: 'Just now',
    comments: [],
  };

  issues.value.unshift(newIssue);
  selectedIssue.value = newIssue;
  showNewIssueModal.value = false;
  newIssueTitle.value = '';
  newIssueDescription.value = '';
  saveLocalIssues();
}

function submitComment() {
  if (!newCommentText.value.trim() || !selectedIssue.value) return;

  selectedIssue.value.comments.push({
    id: `c_${Date.now()}`,
    author: 'You',
    text: newCommentText.value.trim(),
    timestamp: 'Just now',
  });
  newCommentText.value = '';
  saveLocalIssues();
}

function toggleIssueStatus() {
  if (!selectedIssue.value) return;
  if (selectedIssue.value.status === 'open') {
    selectedIssue.value.status = 'closed';
  } else {
    selectedIssue.value.status = 'open';
  }
  selectedIssue.value.updatedAt = 'Just now';
  saveLocalIssues();
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
        <span class="badge badge-branch">{{ openCount }} Open</span>
      </div>

      <div class="header-actions">
        <button class="btn btn-primary btn-xs" @click="showNewIssueModal = true">
          <Plus :size="12" /> New Issue
        </button>
        <button class="btn btn-secondary btn-xs" @click="emit('close')">
          <ArrowLeft :size="12" /> Close View
        </button>
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

            <div class="search-box">
              <Search :size="12" class="search-icon" />
              <input v-model="searchQuery" placeholder="Search issues..." class="search-input" />
            </div>
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
            <button class="btn btn-secondary btn-xs back-btn" @click="selectedIssue = null">
              <ArrowLeft :size="12" /> Back to Issues
            </button>

            <div class="detail-title-row">
              <h2 class="detail-title">{{ selectedIssue.title }}</h2>
              <span class="detail-id">#{{ selectedIssue.id }}</span>
              <span class="badge" :class="selectedIssue.status === 'open' ? 'badge-warning' : 'badge-purple'">
                {{ selectedIssue.status.toUpperCase() }}
              </span>
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
              <button class="btn btn-secondary btn-xs" @click="toggleIssueStatus">
                <CheckCircle2 v-if="selectedIssue.status === 'open'" :size="12" class="icon-success" />
                <AlertCircle v-else :size="12" class="icon-warning" />
                {{ selectedIssue.status === 'open' ? 'Close Issue' : 'Reopen Issue' }}
              </button>
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
                <button class="btn btn-primary btn-xs" @click="submitComment" :disabled="!newCommentText.trim()">
                  <Send :size="11" /> Post Comment
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Create New Issue Modal -->
    <div v-if="showNewIssueModal" class="modal-overlay" @click.self="showNewIssueModal = false">
      <div class="new-issue-modal gk-panel">
        <div class="modal-header">
          <AlertCircle :size="15" class="icon-warning" />
          <span class="title">Create New Issue</span>
          <button class="icon-btn" @click="showNewIssueModal = false">✕</button>
        </div>

        <div class="modal-body">
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
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary btn-xs" @click="showNewIssueModal = false">Cancel</button>
          <button class="btn btn-primary btn-xs" @click="createIssue" :disabled="!newIssueTitle.trim()">
            Create Issue
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.issues-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-dark);
  border-radius: 0;
  border: none;
  overflow: hidden;
}

.issues-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: #121417;
  border-bottom: 1px solid var(--border-color);
}

.header-title-box {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 700;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.issues-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px;
}

.issues-filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
  gap: 12px;
}

.status-tabs {
  display: flex;
  gap: 4px;
  background: rgba(0, 0, 0, 0.3);
  padding: 3px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.status-tab {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.15s ease;
}

.status-tab.active {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-main);
  font-weight: 600;
}

.tab-count {
  font-size: 10px;
  background: rgba(255, 255, 255, 0.1);
  padding: 1px 5px;
  border-radius: 8px;
}

.filter-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.label-filter-wrap {
  display: flex;
  align-items: center;
  gap: 5px;
  background: rgba(0, 0, 0, 0.3);
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.label-select {
  background: transparent;
  border: none;
  color: var(--text-main);
  font-size: 11px;
  outline: none;
  cursor: pointer;
}

.label-select option {
  background-color: #21252b;
  color: #e6e6e6;
  padding: 6px 10px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  width: 220px;
}

.search-input {
  width: 100%;
  padding-left: 28px;
}

.search-icon {
  position: absolute;
  left: 8px;
  color: var(--text-dim);
}

.issues-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.issue-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  cursor: pointer;
}

.issue-card-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.issue-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issue-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.issue-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
}

.issue-id {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.issue-meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-muted);
}

.labels-list {
  display: flex;
  align-items: center;
  gap: 4px;
}

.label-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 9px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  border: 1px solid;
}

.priority-badge {
  font-size: 9px;
  padding: 1px 5px;
}

.issue-card-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.comment-count {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-dim);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px;
  color: var(--text-dim);
  font-size: 12px;
}

/* Issue Detail Layout */
.issue-detail-layout {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.detail-header {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: #181a1f;
  padding: 12px 16px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.back-btn {
  align-self: flex-start;
}

.detail-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.detail-title {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-main);
}

.detail-id {
  font-size: 14px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.detail-meta-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

.detail-actions {
  display: flex;
  gap: 8px;
}

.detail-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.description-card {
  padding: 12px;
}

.card-header-sm {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.card-text {
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-main);
}

.comments-timeline {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
}

.comment-card {
  padding: 10px;
}

.comment-header {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 4px;
}

.comment-author {
  font-weight: 600;
  color: var(--primary);
}

.comment-text {
  font-size: 12px;
  color: var(--text-main);
}

.empty-comments {
  font-size: 11px;
  color: var(--text-dim);
  font-style: italic;
}

.add-comment-box {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.comment-textarea {
  width: 100%;
  resize: none;
  font-size: 12px;
}

.comment-actions {
  display: flex;
  justify-content: flex-end;
}

/* Modal */
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

.new-issue-modal {
  width: 480px;
  background: var(--bg-panel);
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 13px;
}

.modal-header .title {
  flex: 1;
}

.modal-body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.label-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.label-picker-tag {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  border: 1px solid;
  cursor: pointer;
  user-select: none;
}

.label-picker-tag.selected {
  font-weight: 600;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid var(--border-color);
  background: #181a1f;
}

.icon-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
}

.badge-warning {
  background: rgba(255, 165, 2, 0.18);
  color: #ffa502;
  border: 1px solid rgba(255, 165, 2, 0.4);
}

.badge-danger {
  background: rgba(255, 71, 87, 0.18);
  color: #ff4757;
  border: 1px solid rgba(255, 71, 87, 0.4);
}

.badge-purple {
  background: rgba(156, 136, 255, 0.18);
  color: #9c88ff;
  border: 1px solid rgba(156, 136, 255, 0.4);
}

.badge-muted {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
}

.btn-xs {
  padding: 4px 8px;
  font-size: 11px;
}

.dot {
  color: var(--text-dim);
}
</style>
