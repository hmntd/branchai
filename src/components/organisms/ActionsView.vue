<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  Play,
  RefreshCw,
  CheckCircle2,
  XCircle,
  Clock,
  Loader2,
  ExternalLink,
  GitBranch,
  Copy,
  ChevronDown,
  ChevronRight,
  Terminal,
  AlertTriangle,
} from '@lucide/vue';
import BaseButton from '../atoms/BaseButton.vue';
import SearchBox from '../molecules/SearchBox.vue';
import TriggerWorkflowModal from './TriggerWorkflowModal.vue';
import {
  GithubWorkflow,
  GithubWorkflowRun,
  GithubJob,
} from '../../types/actions';

const props = defineProps<{
  repoPath: string;
  currentBranch?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const workflows = ref<GithubWorkflow[]>([]);
const runs = ref<GithubWorkflowRun[]>([]);
const jobs = ref<GithubJob[]>([]);
const logsText = ref<string>('');

const selectedWorkflowId = ref<number | null>(null);
const selectedRun = ref<GithubWorkflowRun | null>(null);
const selectedJobId = ref<number | null>(null);
const expandedJobIds = ref<Set<number>>(new Set());

const loadingWorkflows = ref(false);
const loadingRuns = ref(false);
const loadingJobs = ref(false);
const loadingLogs = ref(false);
const errorMessage = ref('');

const filterText = ref('');
const statusFilter = ref<'all' | 'success' | 'failure' | 'in_progress'>('all');
const logSearchText = ref('');
const showTriggerModal = ref(false);
const copiedLog = ref(false);

const filteredRuns = computed(() => {
  return runs.value.filter((r) => {
    if (statusFilter.value === 'success' && r.conclusion !== 'success') return false;
    if (statusFilter.value === 'failure' && r.conclusion !== 'failure') return false;
    if (
      statusFilter.value === 'in_progress' &&
      r.status !== 'in_progress' &&
      r.status !== 'queued'
    )
      return false;

    if (filterText.value.trim()) {
      const q = filterText.value.toLowerCase();
      const nameMatch = (r.name || '').toLowerCase().includes(q);
      const branchMatch = (r.head_branch || '').toLowerCase().includes(q);
      const eventMatch = (r.event || '').toLowerCase().includes(q);
      const numMatch = r.run_number.toString().includes(q);
      return nameMatch || branchMatch || eventMatch || numMatch;
    }
    return true;
  });
});

const displayedLogs = computed(() => {
  if (!logsText.value) return 'No log output available for this job.';
  if (!logSearchText.value.trim()) return logsText.value;
  const q = logSearchText.value.toLowerCase();
  return logsText.value
    .split('\n')
    .filter((line) => line.toLowerCase().includes(q))
    .join('\n');
});

async function loadWorkflows() {
  if (!props.repoPath) return;
  loadingWorkflows.value = true;
  errorMessage.value = '';
  try {
    const list: GithubWorkflow[] = await invoke('get_github_workflows', {
      repoPath: props.repoPath,
    });
    workflows.value = list;
    await loadRuns();
  } catch (err: any) {
    errorMessage.value = typeof err === 'string' ? err : err.message || 'Failed to load workflows';
  } finally {
    loadingWorkflows.value = false;
  }
}

async function loadRuns() {
  if (!props.repoPath) return;
  loadingRuns.value = true;
  try {
    const list: GithubWorkflowRun[] = await invoke('get_workflow_runs', {
      repoPath: props.repoPath,
      workflowId: selectedWorkflowId.value,
    });
    runs.value = list;

    if (list.length > 0) {
      if (!selectedRun.value || !list.some((r) => r.id === selectedRun.value?.id)) {
        await selectRun(list[0]);
      }
    } else {
      selectedRun.value = null;
      jobs.value = [];
      logsText.value = '';
    }
  } catch (err: any) {
    console.error('Failed to fetch workflow runs:', err);
  } finally {
    loadingRuns.value = false;
  }
}

async function selectRun(run: GithubWorkflowRun) {
  selectedRun.value = run;
  jobs.value = [];
  logsText.value = '';
  selectedJobId.value = null;
  loadingJobs.value = true;

  try {
    const list: GithubJob[] = await invoke('get_workflow_run_jobs', {
      repoPath: props.repoPath,
      runId: run.id,
    });
    jobs.value = list;

    if (list.length > 0) {
      expandedJobIds.value.add(list[0].id);
      await selectJob(list[0].id);
    }
  } catch (err: any) {
    console.error('Failed to fetch jobs for run:', err);
  } finally {
    loadingJobs.value = false;
  }
}

async function selectJob(jobId: number) {
  selectedJobId.value = jobId;
  loadingLogs.value = true;
  logsText.value = '';

  try {
    const text: string = await invoke('get_job_logs', {
      repoPath: props.repoPath,
      jobId,
    });
    logsText.value = text;
  } catch (err: any) {
    logsText.value = `[Error loading logs]: ${typeof err === 'string' ? err : err.message}`;
  } finally {
    loadingLogs.value = false;
  }
}

function toggleJobExpand(jobId: number) {
  if (expandedJobIds.value.has(jobId)) {
    expandedJobIds.value.delete(jobId);
  } else {
    expandedJobIds.value.add(jobId);
  }
}

function handleCopyLogs() {
  if (!logsText.value) return;
  navigator.clipboard.writeText(logsText.value);
  copiedLog.value = true;
  setTimeout(() => (copiedLog.value = false), 1500);
}

function openExternalUrl(url?: string) {
  if (!url) return;
  window.open(url, '_blank');
}

function formatDuration(startStr?: string, endStr?: string) {
  if (!startStr) return '';
  const start = new Date(startStr).getTime();
  const end = endStr ? new Date(endStr).getTime() : Date.now();
  const diffSec = Math.max(0, Math.floor((end - start) / 1000));

  if (diffSec < 60) return `${diffSec}s`;
  const mins = Math.floor(diffSec / 60);
  const secs = diffSec % 60;
  return `${mins}m ${secs}s`;
}

function formatRelativeTime(dateStr: string) {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / (1000 * 60));
  if (mins < 1) return 'Just now';
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

watch(
  () => props.repoPath,
  async () => {
    selectedWorkflowId.value = null;
    selectedRun.value = null;
    await loadWorkflows();
  },
  { immediate: true }
);

watch(selectedWorkflowId, async () => {
  await loadRuns();
});
</script>

<template>
  <div class="actions-container">
    <!-- Header Action Toolbar -->
    <div class="actions-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-title">
          <Play :size="16" class="icon-purple" />
          <span>GitHub Actions & CI/CD Pipelines</span>
        </div>

        <select v-model="selectedWorkflowId" class="select-sm">
          <option :value="null">All Workflows ({{ workflows.length }})</option>
          <option v-for="w in workflows" :key="w.id" :value="w.id">
            {{ w.name }}
          </option>
        </select>
      </div>

      <div class="toolbar-right">
        <BaseButton size="xs" variant="secondary" :disabled="loadingRuns" @click="loadWorkflows">
          <RefreshCw :size="12" :class="{ spinning: loadingRuns || loadingWorkflows }" /> Refresh
        </BaseButton>

        <BaseButton size="xs" variant="primary" :disabled="workflows.length === 0" @click="showTriggerModal = true">
          <Play :size="12" /> Run Workflow
        </BaseButton>
      </div>
    </div>

    <!-- Main Content split 2-pane area -->
    <div class="actions-body">
      <!-- Left Pane: Runs History Sidebar -->
      <div class="runs-sidebar">
        <div class="runs-filter">
          <SearchBox v-model="filterText" placeholder="Filter runs (#, branch, event)..." class="flex-1" />

          <select v-model="statusFilter" class="select-sm">
            <option value="all">All</option>
            <option value="success">Success</option>
            <option value="failure">Failed</option>
            <option value="in_progress">Running</option>
          </select>
        </div>

        <div v-if="loadingRuns && runs.length === 0" class="empty-state">
          <Loader2 :size="24" class="spinning icon-purple" />
          <span>Fetching workflow runs...</span>
        </div>

        <div v-else-if="errorMessage" class="empty-state">
          <AlertTriangle :size="24" class="text-amber-400" />
          <span class="text-amber-300 font-mono text-[11px] max-w-[280px] break-words">{{ errorMessage }}</span>
        </div>

        <div v-else-if="filteredRuns.length === 0" class="empty-state">
          <span>No workflow runs found.</span>
        </div>

        <div v-else class="runs-list">
          <div v-for="run in filteredRuns" :key="run.id" class="run-card"
            :class="{ active: selectedRun?.id === run.id }" @click="selectRun(run)">
            <div class="run-card-header">
              <!-- Status indicator badge -->
              <div class="badge-status" :class="{
                'badge-success': run.conclusion === 'success',
                'badge-failure': run.conclusion === 'failure',
                'badge-progress': run.status === 'in_progress',
                'badge-queued': run.status === 'queued',
                'badge-cancelled': run.conclusion === 'cancelled'
              }">
                <CheckCircle2 v-if="run.conclusion === 'success'" :size="11" />
                <XCircle v-else-if="run.conclusion === 'failure'" :size="11" />
                <Loader2 v-else-if="run.status === 'in_progress'" :size="11" class="spinning" />
                <Clock v-else :size="11" />
                <span>{{ run.conclusion || run.status }}</span>
              </div>

              <span class="run-number">#{{ run.run_number }}</span>
            </div>

            <span class="run-name">{{ run.name || 'Workflow Run' }}</span>

            <div class="run-card-meta">
              <span v-if="run.head_branch" class="meta-item">
                <GitBranch :size="10" /> {{ run.head_branch }}
              </span>
              <span class="meta-item">{{ run.event }}</span>
            </div>

            <div class="run-card-footer">
              <div class="actor-info" v-if="run.actor_login">
                <img v-if="run.actor_avatar_url" :src="run.actor_avatar_url" class="actor-avatar" alt="actor" />
                <span class="actor-name">{{ run.actor_login }}</span>
              </div>

              <span>{{ formatRelativeTime(run.created_at) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Pane: Run Detail, Jobs Tree & Live Terminal Log Viewer -->
      <div class="run-detail-pane">
        <template v-if="selectedRun">
          <div class="detail-header">
            <div class="detail-title">
              <CheckCircle2 v-if="selectedRun.conclusion === 'success'" :size="16" class="text-emerald-400" />
              <XCircle v-else-if="selectedRun.conclusion === 'failure'" :size="16" class="text-red-400" />
              <Loader2 v-else :size="16" class="spinning text-sky-400" />
              <span>{{ selectedRun.name }} #{{ selectedRun.run_number }}</span>
            </div>

            <div class="toolbar-right">
              <BaseButton size="xs" variant="secondary" title="View run on GitHub"
                @click="openExternalUrl(selectedRun.html_url)">
                <ExternalLink :size="12" /> Open on GitHub
              </BaseButton>
            </div>
          </div>

          <div class="detail-body">
            <!-- Jobs & Steps Breakdown -->
            <div class="jobs-section">
              <span class="section-label">JOBS & EXECUTION STEPS ({{ jobs.length }})</span>

              <div v-if="loadingJobs" class="flex items-center gap-2 text-white/50 text-[11px]">
                <Loader2 :size="14" class="spinning icon-purple" /> Loading job tree...
              </div>

              <div v-else class="jobs-grid">
                <div v-for="job in jobs" :key="job.id" class="job-item">
                  <div class="job-header" @click="toggleJobExpand(job.id)">
                    <div class="job-title">
                      <component :is="expandedJobIds.has(job.id) ? ChevronDown : ChevronRight" :size="12" />
                      <CheckCircle2 v-if="job.conclusion === 'success'" :size="13" class="text-emerald-400" />
                      <XCircle v-else-if="job.conclusion === 'failure'" :size="13" class="text-red-400" />
                      <Loader2 v-else :size="13" class="spinning text-sky-400" />
                      <span>{{ job.name }}</span>
                    </div>

                    <div class="flex items-center gap-2">
                      <span class="text-[10px] text-white/40 font-mono">{{ formatDuration(job.started_at,
                        job.completed_at) }}</span>
                      <BaseButton size="xs" variant="secondary" title="Load job logs" @click.stop="selectJob(job.id)">
                        <Terminal :size="11" /> Logs
                      </BaseButton>
                    </div>
                  </div>

                  <!-- Steps Hierarchy -->
                  <div v-if="expandedJobIds.has(job.id)" class="job-steps">
                    <div v-for="step in job.steps" :key="step.number" class="step-item">
                      <div class="step-name">
                        <CheckCircle2 v-if="step.conclusion === 'success'" :size="11" class="text-emerald-400" />
                        <XCircle v-else-if="step.conclusion === 'failure'" :size="11" class="text-red-400" />
                        <Loader2 v-else-if="step.status === 'in_progress'" :size="11" class="spinning text-sky-400" />
                        <Clock v-else :size="11" class="text-white/30" />
                        <span>{{ step.name }}</span>
                      </div>
                      <span class="text-[10px] text-white/40 font-mono">{{ formatDuration(step.started_at,
                        step.completed_at) }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Terminal Log Viewer -->
            <div class="logs-section">
              <div class="logs-header">
                <div class="flex items-center gap-2">
                  <Terminal :size="14" class="icon-purple" />
                  <span>Job Terminal Logs</span>
                  <span v-if="loadingLogs" class="flex items-center gap-1 text-sky-400 text-[10px]">
                    <Loader2 :size="10" class="spinning" /> Streaming...
                  </span>
                </div>

                <div class="logs-actions">
                  <SearchBox v-model="logSearchText" placeholder="Filter log output..." class="w-[180px]" />
                  <BaseButton size="xs" variant="secondary" title="Copy raw log output" @click="handleCopyLogs">
                    <Copy :size="11" /> {{ copiedLog ? 'Copied!' : 'Copy' }}
                  </BaseButton>
                </div>
              </div>

              <pre class="terminal-log-viewer">{{ displayedLogs }}</pre>
            </div>
          </div>
        </template>

        <div v-else class="empty-state">
          <Play :size="32" class="icon-purple/40" />
          <span>Select a workflow run from the left panel to inspect jobs and stream execution logs.</span>
        </div>
      </div>
    </div>

    <!-- Trigger Workflow Dispatch Modal -->
    <TriggerWorkflowModal v-if="showTriggerModal" :repo-path="repoPath" :workflows="workflows"
      :current-branch="currentBranch" @close="showTriggerModal = false" @triggered="loadRuns" />
  </div>
</template>

<style scoped src="../../styles/organisms/ActionsView.css"></style>
