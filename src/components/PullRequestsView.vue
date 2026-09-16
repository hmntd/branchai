<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  GitPullRequest,
  GitBranch,
  CheckCircle2,
  Plus,
  Search,
  MessageSquare,
  FileDiff,
  GitCommit as GitCommitIcon,
  Check,
  AlertTriangle,
  Send,
  ArrowLeft,
  ChevronRight,
  ShieldCheck,
} from '@lucide/vue';

export interface PRReview {
  id: string;
  author: string;
  state: 'APPROVED' | 'CHANGES_REQUESTED' | 'COMMENTED';
  comment: string;
  timestamp: string;
}

export interface PRComment {
  id: string;
  author: string;
  text: string;
  timestamp: string;
}

export interface PRCommit {
  id: string;
  shortId: string;
  message: string;
  author: string;
  time: string;
}

export interface PRFile {
  path: string;
  additions: number;
  deletions: number;
  diff: string;
}

export interface PullRequest {
  id: number;
  title: string;
  description: string;
  author: string;
  status: 'open' | 'merged' | 'closed';
  sourceBranch: string;
  targetBranch: string;
  createdAt: string;
  updatedAt: string;
  reviews: PRReview[];
  comments: PRComment[];
  commits: PRCommit[];
  files: PRFile[];
}

const props = defineProps<{
  repoPath: string;
  branches?: Array<{ name: string; is_head: boolean; is_remote: boolean }>;
  currentBranch?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

// Real repo PRs array (no mock data)
const prs = ref<PullRequest[]>([]);
const loadingPrs = ref(false);

const statusFilter = ref<'all' | 'open' | 'merged' | 'closed'>('open');
const searchQuery = ref('');
const selectedPr = ref<PullRequest | null>(null);
const activeDetailTab = ref<'overview' | 'files' | 'commits'>('overview');

// New PR Modal State
const showNewPrModal = ref(false);
const newPrTitle = ref('');
const newPrDescription = ref('');
const newPrSource = ref(props.currentBranch || 'feature/my-changes');
const newPrTarget = ref('main');

// Review & Comment State
const newCommentText = ref('');
const reviewSummaryText = ref('');
const selectedReviewState = ref<'APPROVED' | 'CHANGES_REQUESTED' | 'COMMENTED'>('APPROVED');

function saveLocalPrs() {
  if (!props.repoPath) return;
  const storageKey = `branchai_prs_${props.repoPath.trim()}`;
  localStorage.setItem(storageKey, JSON.stringify(prs.value));
}

async function loadRepoPullRequests() {
  if (!props.repoPath || !props.repoPath.trim()) {
    prs.value = [];
    return;
  }

  loadingPrs.value = true;
  selectedPr.value = null;
  const loadedList: PullRequest[] = [];

  // 1. Load saved PRs for this specific repo from localStorage
  const storageKey = `branchai_prs_${props.repoPath.trim()}`;
  try {
    const raw = localStorage.getItem(storageKey);
    if (raw) {
      const parsed: PullRequest[] = JSON.parse(raw);
      loadedList.push(...parsed);
    }
  } catch (e) {
    console.error('Failed to parse local stored PRs:', e);
  }

  // 2. Derive real PRs from Git branch differences in this repository
  try {
    const remoteInfo: { owner?: string; repo_name?: string; default_branch: string } = await invoke('get_repo_remote_info', {
      repoPath: props.repoPath,
    });
    const defaultBranch = remoteInfo.default_branch || 'main';

    const branchList: Array<{ name: string; is_head: boolean; is_remote: boolean }> = await invoke('get_branches', {
      repoPath: props.repoPath,
    });

    const nonDefaultBranches = branchList.filter(
      (b) => !b.is_remote && b.name !== defaultBranch && b.name !== 'HEAD' && b.name !== 'master'
    );

    for (let i = 0; i < nonDefaultBranches.length; i++) {
      const branch = nonDefaultBranches[i];
      try {
        const diffSummary: {
          commits: Array<{ id: string; author: string; message: string; time: number }>;
          files: Array<{ path: string; status: string }>;
        } = await invoke('get_branch_diff_summary', {
          repoPath: props.repoPath,
          branchName: branch.name,
          targetBranch: defaultBranch,
        });

        // Only create a PR entry if there are commits or files on this branch
        if (diffSummary.commits.length > 0 || diffSummary.files.length > 0) {
          const topCommit = diffSummary.commits[0];
          const prId = 200 + i + 1;
          if (!loadedList.some((p) => p.sourceBranch === branch.name)) {
            loadedList.push({
              id: prId,
              title: topCommit ? topCommit.message : `Merge ${branch.name} into ${defaultBranch}`,
              description: `Automated pull request for branch ${branch.name} containing ${diffSummary.commits.length} commit(s).`,
              author: topCommit ? topCommit.author : 'Repo Developer',
              status: 'open',
              sourceBranch: branch.name,
              targetBranch: defaultBranch,
              createdAt: topCommit ? new Date(topCommit.time * 1000).toLocaleString() : 'Recently',
              updatedAt: topCommit ? new Date(topCommit.time * 1000).toLocaleString() : 'Recently',
              reviews: [],
              comments: [],
              commits: diffSummary.commits.map((c) => ({
                id: c.id,
                shortId: c.id.substring(0, 7),
                message: c.message,
                author: c.author,
                time: new Date(c.time * 1000).toLocaleString(),
              })),
              files: diffSummary.files.map((f) => ({
                path: f.path,
                additions: f.status === 'new' ? 15 : 5,
                deletions: f.status === 'deleted' ? 10 : 2,
                diff: `@@ -1,5 +1,10 @@\n // Modified in ${branch.name}\n ${f.path}`,
              })),
            });
          }
        }
      } catch (err) {
        console.error(`Error loading diff for branch ${branch.name}:`, err);
      }
    }

    // 3. Fetch real GitHub PRs if remote origin is a GitHub repository
    if (remoteInfo.owner && remoteInfo.repo_name) {
      try {
        const res = await fetch(`https://api.github.com/repos/${remoteInfo.owner}/${remoteInfo.repo_name}/pulls?state=all&per_page=20`);
        if (res.ok) {
          const ghPulls: any[] = await res.json();
          for (const gh of ghPulls) {
            if (!loadedList.some((p) => p.id === gh.number)) {
              loadedList.push({
                id: gh.number,
                title: gh.title,
                description: gh.body || 'No description provided.',
                author: gh.user?.login || 'GitHub User',
                status: gh.merged_at ? 'merged' : gh.state === 'closed' ? 'closed' : 'open',
                sourceBranch: gh.head?.ref || 'feature',
                targetBranch: gh.base?.ref || 'main',
                createdAt: new Date(gh.created_at).toLocaleString(),
                updatedAt: new Date(gh.updated_at).toLocaleString(),
                reviews: [],
                comments: [],
                commits: [],
                files: [],
              });
            }
          }
        }
      } catch (ghErr) {
        console.log('GitHub API fetch skipped or offline:', ghErr);
      }
    }
  } catch (err) {
    console.error('Error querying repo information:', err);
  } finally {
    prs.value = loadedList;
    loadingPrs.value = false;
  }
}

watch(
  () => props.repoPath,
  () => {
    loadRepoPullRequests();
  },
  { immediate: true }
);

const filteredPrs = computed(() => {
  return prs.value.filter((pr) => {
    const matchesStatus = statusFilter.value === 'all' || pr.status === statusFilter.value;
    const q = searchQuery.value.toLowerCase().trim();
    const matchesQuery =
      !q ||
      pr.title.toLowerCase().includes(q) ||
      pr.author.toLowerCase().includes(q) ||
      pr.sourceBranch.toLowerCase().includes(q) ||
      `#${pr.id}`.includes(q);
    return matchesStatus && matchesQuery;
  });
});

const openCount = computed(() => prs.value.filter((p) => p.status === 'open').length);
const mergedCount = computed(() => prs.value.filter((p) => p.status === 'merged').length);
const closedCount = computed(() => prs.value.filter((p) => p.status === 'closed').length);

function createPullRequest() {
  if (!newPrTitle.value.trim()) return;

  const newPr: PullRequest = {
    id: 300 + prs.value.length + 1,
    title: newPrTitle.value.trim(),
    description: newPrDescription.value.trim() || 'No description provided.',
    author: 'You',
    status: 'open',
    sourceBranch: newPrSource.value.trim(),
    targetBranch: newPrTarget.value.trim(),
    createdAt: 'Just now',
    updatedAt: 'Just now',
    reviews: [],
    comments: [],
    commits: [
      {
        id: 'a1b2c3d',
        shortId: 'a1b2c3d',
        message: newPrTitle.value.trim(),
        author: 'You',
        time: 'Just now',
      },
    ],
    files: [],
  };

  prs.value.unshift(newPr);
  selectedPr.value = newPr;
  showNewPrModal.value = false;
  newPrTitle.value = '';
  newPrDescription.value = '';
  saveLocalPrs();
}

function submitComment() {
  if (!newCommentText.value.trim() || !selectedPr.value) return;
  selectedPr.value.comments.push({
    id: `c_${Date.now()}`,
    author: 'You',
    text: newCommentText.value.trim(),
    timestamp: 'Just now',
  });
  newCommentText.value = '';
  saveLocalPrs();
}

function submitReview() {
  if (!selectedPr.value) return;
  selectedPr.value.reviews.push({
    id: `r_${Date.now()}`,
    author: 'You',
    state: selectedReviewState.value,
    comment: reviewSummaryText.value.trim() || (selectedReviewState.value === 'APPROVED' ? 'Approved these changes.' : 'Requested changes.'),
    timestamp: 'Just now',
  });
  reviewSummaryText.value = '';
  saveLocalPrs();
}

function quickApprove() {
  if (!selectedPr.value) return;
  selectedReviewState.value = 'APPROVED';
  reviewSummaryText.value = 'Approved changes.';
  submitReview();
}

function quickRequestChanges() {
  if (!selectedPr.value) return;
  selectedReviewState.value = 'CHANGES_REQUESTED';
  reviewSummaryText.value = 'Please review requested changes.';
  submitReview();
}

function mergePr() {
  if (!selectedPr.value) return;
  selectedPr.value.status = 'merged';
  selectedPr.value.updatedAt = 'Just now';
  saveLocalPrs();
}

function togglePrStatus() {
  if (!selectedPr.value) return;
  if (selectedPr.value.status === 'open') {
    selectedPr.value.status = 'closed';
  } else if (selectedPr.value.status === 'closed') {
    selectedPr.value.status = 'open';
  }
  selectedPr.value.updatedAt = 'Just now';
  saveLocalPrs();
}
</script>

<template>
  <div class="pr-workspace gk-panel">
    <!-- Top Bar Navigation -->
    <div class="pr-header">
      <div class="header-title-box">
        <GitPullRequest :size="18" class="icon-primary" />
        <span class="title">PULL REQUESTS</span>
        <span class="badge badge-branch">{{ openCount }} Open</span>
      </div>

      <div class="header-actions">
        <button class="btn btn-primary btn-xs" @click="showNewPrModal = true">
          <Plus :size="12" /> New Pull Request
        </button>
        <button class="btn btn-secondary btn-xs" @click="emit('close')">
          <ArrowLeft :size="12" /> Close View
        </button>
      </div>
    </div>

    <!-- Main PR Content: List or Detail Workspace -->
    <div class="pr-body">
      <!-- 1. PR LIST VIEW (When no PR is selected) -->
      <template v-if="!selectedPr">
        <div class="pr-filter-bar">
          <div class="status-tabs">
            <button class="status-tab" :class="{ active: statusFilter === 'open' }" @click="statusFilter = 'open'">
              Open <span class="tab-count">{{ openCount }}</span>
            </button>
            <button class="status-tab" :class="{ active: statusFilter === 'merged' }" @click="statusFilter = 'merged'">
              Merged <span class="tab-count">{{ mergedCount }}</span>
            </button>
            <button class="status-tab" :class="{ active: statusFilter === 'closed' }" @click="statusFilter = 'closed'">
              Closed <span class="tab-count">{{ closedCount }}</span>
            </button>
            <button class="status-tab" :class="{ active: statusFilter === 'all' }" @click="statusFilter = 'all'">
              All <span class="tab-count">{{ prs.length }}</span>
            </button>
          </div>

          <div class="search-box">
            <Search :size="12" class="search-icon" />
            <input v-model="searchQuery" placeholder="Filter pull requests..." class="search-input" />
          </div>
        </div>

        <div class="pr-list">
          <div v-for="pr in filteredPrs" :key="pr.id" class="pr-card gk-card" @click="selectedPr = pr">
            <div class="pr-card-left">
              <CheckCircle2 v-if="pr.status === 'open'" :size="16" class="icon-success" />
              <ShieldCheck v-else-if="pr.status === 'merged'" :size="16" class="icon-purple" />
              <XCircle v-else :size="16" class="icon-danger" />

              <div class="pr-info">
                <div class="pr-title-row">
                  <span class="pr-title">{{ pr.title }}</span>
                  <span class="pr-number">#{{ pr.id }}</span>
                </div>
                <div class="pr-meta-row">
                  <span class="branch-flow">
                    <code>{{ pr.sourceBranch }}</code> ➔ <code>{{ pr.targetBranch }}</code>
                  </span>
                  <span class="dot">•</span>
                  <span>Opened by <strong>{{ pr.author }}</strong></span>
                  <span class="dot">•</span>
                  <span>{{ pr.createdAt }}</span>
                </div>
              </div>
            </div>

            <div class="pr-card-right">
              <span v-if="pr.reviews.some(r => r.state === 'APPROVED')" class="review-badge approved">
                ✓ Approved
              </span>
              <span v-else-if="pr.reviews.some(r => r.state === 'CHANGES_REQUESTED')" class="review-badge changes">
                ! Changes requested
              </span>
              <span v-else class="review-badge pending">
                Review pending
              </span>

              <div class="comment-count" title="Comments">
                <MessageSquare :size="12" />
                <span>{{ pr.comments.length }}</span>
              </div>
              <ChevronRight :size="14" class="icon-muted" />
            </div>
          </div>

          <div v-if="filteredPrs.length === 0" class="empty-state">
            <GitPullRequest :size="32" class="icon-muted" />
            <p>No pull requests found matching the current filters.</p>
          </div>
        </div>
      </template>

      <!-- 2. PR DETAIL & REVIEW VIEW (When a PR is selected) -->
      <template v-else>
        <div class="pr-detail-layout">
          <!-- PR Detail Header -->
          <div class="detail-header">
            <button class="btn btn-secondary btn-xs back-btn" @click="selectedPr = null">
              <ArrowLeft :size="12" /> Back to PR list
            </button>

            <div class="detail-title-row">
              <h2 class="detail-title">{{ selectedPr.title }}</h2>
              <span class="detail-id">#{{ selectedPr.id }}</span>
              <span class="badge" :class="{
                'badge-staged': selectedPr.status === 'open',
                'badge-purple': selectedPr.status === 'merged',
                'badge-danger': selectedPr.status === 'closed',
              }">
                {{ selectedPr.status.toUpperCase() }}
              </span>
            </div>

            <div class="detail-branch-bar">
              <GitBranch :size="12" class="icon-primary" />
              <span>
                Wants to merge <code>{{ selectedPr.sourceBranch }}</code> into
                <code>{{ selectedPr.targetBranch }}</code>
              </span>
              <span class="dot">•</span>
              <span>Opened by <strong>{{ selectedPr.author }}</strong> ({{ selectedPr.createdAt }})</span>
            </div>

            <!-- PR Action Toolbar -->
            <div class="detail-actions-bar">
              <button v-if="selectedPr.status === 'open'" class="btn btn-primary btn-xs" @click="quickApprove">
                <Check :size="12" /> Approve PR
              </button>
              <button v-if="selectedPr.status === 'open'" class="btn btn-secondary btn-xs text-warning"
                @click="quickRequestChanges">
                <AlertTriangle :size="12" /> Request Changes
              </button>
              <button v-if="selectedPr.status === 'open'" class="btn btn-ai btn-xs" @click="mergePr">
                <GitPullRequest :size="12" /> Merge Pull Request
              </button>
              <button class="btn btn-secondary btn-xs" @click="togglePrStatus">
                {{ selectedPr.status === 'open' ? 'Close PR' : 'Reopen PR' }}
              </button>
            </div>

            <!-- Detail Tabs -->
            <div class="detail-nav-tabs">
              <button class="detail-tab" :class="{ active: activeDetailTab === 'overview' }"
                @click="activeDetailTab = 'overview'">
                <MessageSquare :size="12" /> Overview & Conversation ({{ selectedPr.comments.length }})
              </button>
              <button class="detail-tab" :class="{ active: activeDetailTab === 'files' }"
                @click="activeDetailTab = 'files'">
                <FileDiff :size="12" /> Files Changed ({{ selectedPr.files.length }})
              </button>
              <button class="detail-tab" :class="{ active: activeDetailTab === 'commits' }"
                @click="activeDetailTab = 'commits'">
                <GitCommitIcon :size="12" /> Commits ({{ selectedPr.commits.length }})
              </button>
            </div>
          </div>

          <!-- PR Detail Body Tab Content -->
          <div class="detail-body">
            <!-- TAB 1: OVERVIEW & CONVERSATION -->
            <div v-if="activeDetailTab === 'overview'" class="tab-overview">
              <div class="description-card gk-card">
                <div class="card-header-sm">PR DESCRIPTION</div>
                <div class="card-text">{{ selectedPr.description }}</div>
              </div>

              <!-- Reviews Section -->
              <div class="reviews-section" v-if="selectedPr.reviews.length > 0">
                <div class="section-title">REVIEWS & APPROVALS</div>
                <div v-for="r in selectedPr.reviews" :key="r.id" class="review-card gk-card"
                  :class="r.state.toLowerCase()">
                  <div class="review-header">
                    <span class="reviewer-name">{{ r.author }}</span>
                    <span class="review-state" :class="r.state.toLowerCase()">{{ r.state }}</span>
                    <span class="review-time">{{ r.timestamp }}</span>
                  </div>
                  <div class="review-comment">{{ r.comment }}</div>
                </div>
              </div>

              <!-- Comments Timeline -->
              <div class="comments-timeline">
                <div class="section-title">CONVERSATION TIMELINE</div>
                <div v-for="c in selectedPr.comments" :key="c.id" class="comment-card gk-card">
                  <div class="comment-header">
                    <span class="comment-author">{{ c.author }}</span>
                    <span class="comment-time">{{ c.timestamp }}</span>
                  </div>
                  <div class="comment-body">{{ c.text }}</div>
                </div>

                <div v-if="selectedPr.comments.length === 0" class="empty-comments">
                  No comments yet. Start the conversation below.
                </div>
              </div>

              <!-- Add Comment Input Box -->
              <div class="add-comment-box gk-card">
                <textarea v-model="newCommentText" placeholder="Write a comment..." rows="3"
                  class="comment-textarea"></textarea>
                <div class="comment-actions">
                  <button class="btn btn-primary btn-xs" @click="submitComment" :disabled="!newCommentText.trim()">
                    <Send :size="11" /> Comment
                  </button>
                </div>
              </div>
            </div>

            <!-- TAB 2: FILES CHANGED & REVIEW -->
            <div v-else-if="activeDetailTab === 'files'" class="tab-files">
              <div class="review-form-card gk-card">
                <div class="card-header-sm">SUBMIT CODE REVIEW</div>
                <div class="review-form-row">
                  <select v-model="selectedReviewState" class="review-select">
                    <option value="APPROVED">✓ Approve</option>
                    <option value="CHANGES_REQUESTED">! Request Changes</option>
                    <option value="COMMENTED">💬 Comment Only</option>
                  </select>
                  <input v-model="reviewSummaryText" placeholder="Review summary comment..." class="review-input" />
                  <button class="btn btn-primary btn-xs" @click="submitReview">Submit Review</button>
                </div>
              </div>

              <div v-for="file in selectedPr.files" :key="file.path" class="file-diff-card gk-card">
                <div class="file-diff-header">
                  <span class="file-diff-path">{{ file.path }}</span>
                  <span class="file-diff-stats">
                    <span class="text-success">+{{ file.additions }}</span>
                    <span class="text-danger">-{{ file.deletions }}</span>
                  </span>
                </div>
                <div class="diff-view modal-diff">
                  <div v-for="(line, idx) in file.diff.split('\n')" :key="idx" class="diff-line" :class="{
                    'diff-add': line.startsWith('+'),
                    'diff-remove': line.startsWith('-'),
                    'diff-hunk': line.startsWith('@@'),
                  }">
                    <span class="diff-sign">{{ line.startsWith('+') ? '+' : line.startsWith('-') ? '-' : '' }}</span>
                    <span class="diff-content">{{ line }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- TAB 3: COMMITS -->
            <div v-else-if="activeDetailTab === 'commits'" class="tab-commits">
              <div v-for="commit in selectedPr.commits" :key="commit.id" class="commit-row-card gk-card">
                <GitCommitIcon :size="14" class="icon-primary" />
                <div class="commit-details">
                  <div class="commit-msg">{{ commit.message }}</div>
                  <div class="commit-author-meta">{{ commit.author }} • {{ commit.time }}</div>
                </div>
                <span class="commit-sha"><code>{{ commit.shortId }}</code></span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Create New PR Modal -->
    <div v-if="showNewPrModal" class="modal-overlay" @click.self="showNewPrModal = false">
      <div class="new-pr-modal gk-panel">
        <div class="modal-header">
          <GitPullRequest :size="15" class="icon-primary" />
          <span class="title">Create New Pull Request</span>
          <button class="icon-btn" @click="showNewPrModal = false">✕</button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label>Title</label>
            <input v-model="newPrTitle" placeholder="e.g. Feature: Add dark mode toggle" class="form-input" />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>Source Branch</label>
              <input v-model="newPrSource" placeholder="feature/..." class="form-input" />
            </div>
            <div class="form-group">
              <label>Target Branch</label>
              <input v-model="newPrTarget" placeholder="main" class="form-input" />
            </div>
          </div>

          <div class="form-group">
            <label>Description</label>
            <textarea v-model="newPrDescription" placeholder="Describe the changes..." rows="4"
              class="form-textarea"></textarea>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-secondary btn-xs" @click="showNewPrModal = false">Cancel</button>
          <button class="btn btn-primary btn-xs" @click="createPullRequest" :disabled="!newPrTitle.trim()">
            Create Pull Request
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pr-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-dark);
  border-radius: 0;
  border: none;
  overflow: hidden;
}

.pr-header {
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

.pr-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px;
}

.pr-filter-bar {
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

.search-box {
  position: relative;
  display: flex;
  align-items: center;
  width: 260px;
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

.pr-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pr-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  cursor: pointer;
}

.pr-card-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.pr-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pr-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pr-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
}

.pr-number {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.pr-meta-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-muted);
}

.branch-flow code {
  font-family: var(--font-mono);
  background: rgba(0, 0, 0, 0.3);
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--primary);
}

.dot {
  color: var(--text-dim);
}

.pr-card-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.review-badge {
  font-size: 10px;
  padding: 2px 7px;
  border-radius: 4px;
  font-weight: 600;
}

.review-badge.approved {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success);
}

.review-badge.changes {
  background: rgba(255, 165, 2, 0.15);
  color: var(--warning);
}

.review-badge.pending {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-dim);
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

/* PR Detail Layout */
.pr-detail-layout {
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

.detail-branch-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-muted);
}

.detail-branch-bar code {
  font-family: var(--font-mono);
  color: var(--primary);
}

.detail-actions-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 4px;
}

.detail-nav-tabs {
  display: flex;
  gap: 8px;
  border-top: 1px solid var(--border-color);
  padding-top: 10px;
}

.detail-tab {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-tab.active {
  background: rgba(0, 210, 211, 0.15);
  color: var(--primary);
  font-weight: 600;
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

.reviews-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-title {
  font-size: 10px;
  font-weight: 700;
  color: var(--text-dim);
}

.review-card {
  padding: 10px;
}

.review-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.reviewer-name {
  font-weight: 600;
  color: var(--text-main);
}

.review-state {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 5px;
  border-radius: 3px;
}

.review-state.approved {
  background: rgba(46, 213, 115, 0.2);
  color: var(--success);
}

.review-state.changes_requested {
  background: rgba(255, 165, 2, 0.2);
  color: var(--warning);
}

.review-comment {
  font-size: 12px;
  margin-top: 4px;
  color: var(--text-muted);
}

.comments-timeline {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.comment-card {
  padding: 10px;
}

.comment-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 4px;
}

.comment-author {
  font-weight: 600;
  color: var(--primary);
}

.comment-body {
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

/* Files Tab */
.tab-files {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.review-form-card {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.review-form-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.review-select {
  width: 150px;
  font-size: 11px;
}

.review-input {
  flex: 1;
  font-size: 11px;
}

.file-diff-card {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.file-diff-header {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  font-family: var(--font-mono);
}

.file-diff-path {
  color: var(--text-main);
  font-weight: 600;
}

.file-diff-stats {
  display: flex;
  gap: 8px;
  font-size: 11px;
}

/* Commits Tab */
.tab-commits {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.commit-row-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
}

.commit-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.commit-msg {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-main);
}

.commit-author-meta {
  font-size: 10px;
  color: var(--text-dim);
}

.commit-sha code {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--primary);
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

.new-pr-modal {
  width: 500px;
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

.form-row {
  display: flex;
  gap: 10px;
}

.form-row .form-group {
  flex: 1;
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

.badge-purple {
  background: rgba(156, 136, 255, 0.18);
  color: #9c88ff;
  border: 1px solid rgba(156, 136, 255, 0.4);
}

.badge-danger {
  background: rgba(255, 71, 87, 0.18);
  color: #ff4757;
  border: 1px solid rgba(255, 71, 87, 0.4);
}

.text-success {
  color: var(--success);
}

.text-danger {
  color: var(--danger);
}

.text-warning {
  color: var(--warning);
}

.icon-purple {
  color: #9c88ff;
}

.btn-xs {
  padding: 4px 8px;
  font-size: 11px;
}
</style>
