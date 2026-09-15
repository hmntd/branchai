<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  FileText,
  Plus,
  Minus,
  ChevronDown,
  ChevronRight,
  Sparkles,
  Check,
  AlertCircle,
  RefreshCw,
  FolderTree,
  ListFilter,
  CheckCircle2,
} from '@lucide/vue';

export interface FileStatus {
  path: string;
  status: string;
  staged: boolean;
}

const props = defineProps<{
  repoPath: string;
  files: FileStatus[];
  currentBranch: string;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
}>();

const selectedFile = ref<FileStatus | null>(null);
const diffText = ref<string>('');
const loadingDiff = ref<boolean>(false);
const showDiffModal = ref<boolean>(false);

const commitSummary = ref('');
const commitDescription = ref('');
const amendCommit = ref(false);

const isGeneratingAi = ref(false);
const isCommitting = ref(false);
const statusMsg = ref('');
const statusError = ref(false);

const isUnstagedOpen = ref(true);
const isStagedOpen = ref(true);

const stagedFiles = computed(() => props.files.filter((f) => f.staged));
const unstagedFiles = computed(() => props.files.filter((f) => !f.staged));

const summaryCharCount = computed(() => 72 - commitSummary.value.length);

async function inspectFileDiff(file: FileStatus) {
  selectedFile.value = file;
  showDiffModal.value = true;
  loadingDiff.value = true;

  try {
    const diff: string = await invoke('get_file_diff', {
      repoPath: props.repoPath,
      filePath: file.path,
      staged: file.staged,
    });
    diffText.value = diff;
  } catch (e) {
    diffText.value = 'Failed to load file diff.';
  } finally {
    loadingDiff.value = false;
  }
}

async function stageFile(file: FileStatus) {
  try {
    await invoke('stage_file', { repoPath: props.repoPath, filePath: file.path });
    emit('refresh');
  } catch (e) {
    console.error(e);
  }
}

async function unstageFile(file: FileStatus) {
  try {
    await invoke('unstage_file', { repoPath: props.repoPath, filePath: file.path });
    emit('refresh');
  } catch (e) {
    console.error(e);
  }
}

async function stageAll() {
  try {
    await invoke('stage_all', { repoPath: props.repoPath });
    emit('refresh');
  } catch (e) {
    console.error(e);
  }
}

async function composeAiCommit() {
  if (stagedFiles.value.length === 0) {
    await stageAll();
  }

  isGeneratingAi.value = true;
  statusMsg.value = '';
  statusError.value = false;

  try {
    const stagedDiff: string = await invoke('get_all_staged_diff', { repoPath: props.repoPath });
    if (!stagedDiff.trim()) {
      statusMsg.value = 'No diff found to compose commit message.';
      statusError.value = true;
      return;
    }

    const aiMsg: string = await invoke('generate_commit_message', { diff: stagedDiff });
    const lines = aiMsg.trim().split('\n');
    commitSummary.value = lines[0] || '';
    commitDescription.value = lines.slice(1).join('\n').trim();
    statusMsg.value = 'AI commit composed!';
  } catch (err: any) {
    statusMsg.value = typeof err === 'string' ? err : err.message || 'AI generation failed.';
    statusError.value = true;
  } finally {
    isGeneratingAi.value = false;
  }
}

async function executeCommit() {
  const fullMsg = commitDescription.value.trim()
    ? `${commitSummary.value.trim()}\n\n${commitDescription.value.trim()}`
    : commitSummary.value.trim();

  if (!fullMsg) {
    statusMsg.value = 'Please provide a commit summary.';
    statusError.value = true;
    return;
  }

  if (stagedFiles.value.length === 0) {
    statusMsg.value = 'Please stage changes before committing.';
    statusError.value = true;
    return;
  }

  isCommitting.value = true;
  statusMsg.value = '';

  try {
    await invoke('create_commit', { repoPath: props.repoPath, message: fullMsg });
    commitSummary.value = '';
    commitDescription.value = '';
    statusMsg.value = 'Committed successfully!';
    statusError.value = false;
    emit('refresh');
  } catch (err: any) {
    statusMsg.value = typeof err === 'string' ? err : err.message || 'Commit failed.';
    statusError.value = true;
  } finally {
    isCommitting.value = false;
  }
}

function getFileIconClass(status: string) {
  if (status.includes('new') || status === 'untracked') return 'text-success';
  if (status.includes('deleted')) return 'text-danger';
  return 'text-warning';
}

function getFilePrefix(status: string) {
  if (status.includes('new') || status === 'untracked') return '+';
  if (status.includes('deleted')) return '-';
  return '✎';
}
</script>

<template>
  <aside class="right-staging-panel gk-panel">
    <!-- Header with Changes Summary -->
    <div class="panel-header">
      <span class="changes-title">
        {{ files.length }} file changes on
        <span class="badge badge-branch">{{ currentBranch || 'master' }}</span>
      </span>

      <div class="view-toggles">
        <button class="btn btn-secondary btn-xs active"><ListFilter :size="11" /> Path</button>
        <button class="btn btn-secondary btn-xs"><FolderTree :size="11" /> Tree</button>
      </div>
    </div>

    <!-- Files Section: Unstaged & Staged -->
    <div class="files-container">
      <!-- Unstaged Files Accordion -->
      <div class="accordion-section">
        <div class="acc-header" @click="isUnstagedOpen = !isUnstagedOpen">
          <component :is="isUnstagedOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="acc-title">Unstaged Files ({{ unstagedFiles.length }})</span>
          <button v-if="unstagedFiles.length > 0" class="btn btn-primary btn-xs stage-all-btn" @click.stop="stageAll">
            Stage All Changes
          </button>
        </div>

        <div v-if="isUnstagedOpen" class="file-list">
          <div
            v-for="file in unstagedFiles"
            :key="file.path"
            class="file-row"
            @click="inspectFileDiff(file)"
          >
            <span class="status-prefix" :class="getFileIconClass(file.status)">{{ getFilePrefix(file.status) }}</span>
            <span class="file-path">{{ file.path }}</span>
            <button class="stage-btn" @click.stop="stageFile(file)" title="Stage File">
              <Plus :size="11" />
            </button>
          </div>
          <div v-if="unstagedFiles.length === 0" class="empty-list">No unstaged changes</div>
        </div>
      </div>

      <!-- Staged Files Accordion -->
      <div class="accordion-section">
        <div class="acc-header" @click="isStagedOpen = !isStagedOpen">
          <component :is="isStagedOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="acc-title">Staged Files ({{ stagedFiles.length }})</span>
        </div>

        <div v-if="isStagedOpen" class="file-list">
          <div
            v-for="file in stagedFiles"
            :key="file.path"
            class="file-row staged"
            @click="inspectFileDiff(file)"
          >
            <span class="status-prefix text-success">✓</span>
            <span class="file-path">{{ file.path }}</span>
            <button class="stage-btn" @click.stop="unstageFile(file)" title="Unstage File">
              <Minus :size="11" />
            </button>
          </div>
          <div v-if="stagedFiles.length === 0" class="empty-list">No staged changes</div>
        </div>
      </div>
    </div>

    <!-- GitKraken Commit Composer Box -->
    <div class="commit-box gk-panel">
      <div class="commit-box-header">
        <span class="commit-icon-label">-o- Commit</span>
      </div>

      <label class="checkbox-label">
        <input type="checkbox" v-model="amendCommit" />
        <span>Amend previous commit</span>
      </label>

      <div class="inputs-group">
        <div class="summary-wrap">
          <input
            v-model="commitSummary"
            placeholder="Commit summary"
            class="summary-input"
          />
          <span class="char-count" :class="{ 'text-danger': summaryCharCount < 0 }">
            {{ summaryCharCount }}
          </span>
        </div>
        <textarea
          v-model="commitDescription"
          placeholder="Description"
          rows="3"
          class="desc-textarea"
        ></textarea>
      </div>

      <div v-if="statusMsg" class="status-alert" :class="statusError ? 'alert-danger' : 'alert-success'">
        <AlertCircle v-if="statusError" :size="12" />
        <Check v-else :size="12" />
        <span>{{ statusMsg }}</span>
      </div>

      <div class="actions-group">
        <!-- Purple "Compose commits with AI" button -->
        <button
          @click="composeAiCommit"
          class="btn btn-ai btn-full"
          :disabled="isGeneratingAi"
        >
          <RefreshCw v-if="isGeneratingAi" :size="13" class="spinning" />
          <Sparkles v-else :size="13" />
          + Compose commits with AI
        </button>

        <!-- Green Stage & Commit Button -->
        <button
          @click="executeCommit"
          class="btn btn-primary btn-full stage-commit-btn"
          :disabled="isCommitting || !commitSummary.trim() || stagedFiles.length === 0"
        >
          <CheckCircle2 :size="13" />
          {{ isCommitting ? 'Committing...' : `-o- Stage Changes to Commit (${stagedFiles.length})` }}
        </button>
      </div>
    </div>

    <!-- File Diff Modal -->
    <div v-if="showDiffModal" class="modal-overlay" @click.self="showDiffModal = false">
      <div class="diff-modal gk-panel">
        <div class="modal-header">
          <FileText :size="15" class="icon-primary" />
          <span class="title">{{ selectedFile?.path }}</span>
          <button class="icon-btn" @click="showDiffModal = false">✕</button>
        </div>
        <div class="modal-body">
          <div v-if="loadingDiff" class="loading">Loading diff...</div>
          <div v-else class="diff-view">
            <div
              v-for="(line, idx) in diffText.split('\n')"
              :key="idx"
              class="diff-line"
              :class="{
                'diff-add': line.startsWith('+'),
                'diff-remove': line.startsWith('-'),
                'diff-header': line.startsWith('@'),
              }"
            >
              {{ line }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.right-staging-panel {
  width: 310px;
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-radius: 0;
  border-left: 1px solid var(--border-color);
  user-select: none;
  overflow: hidden;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-color);
  background: #121417;
}

.changes-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.view-toggles {
  display: flex;
  gap: 3px;
}

.files-container {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 6px;
}

.accordion-section {
  display: flex;
  flex-direction: column;
}

.acc-header {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 6px;
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  cursor: pointer;
}

.acc-title {
  flex: 1;
}

.stage-all-btn {
  background: #2ed573;
  color: #0d1117;
  font-weight: 700;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-left: 8px;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 6px;
  border-radius: 3px;
  font-size: 11px;
  font-family: var(--font-mono);
  cursor: pointer;
  background: rgba(255, 255, 255, 0.02);
}

.file-row:hover {
  background: var(--bg-card-hover);
}

.status-prefix {
  font-weight: 700;
  width: 10px;
}

.file-path {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-main);
}

.stage-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 4px;
}
.stage-btn:hover {
  color: var(--text-main);
}

.empty-list {
  font-size: 10px;
  color: var(--text-dim);
  font-style: italic;
  padding: 4px 6px;
}

/* Commit Box */
.commit-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  background: #181a1f;
  border-radius: 0;
  border-top: 1px solid var(--border-color);
}

.commit-box-header {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  color: var(--text-muted);
  cursor: pointer;
}

.inputs-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.summary-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.summary-input {
  width: 100%;
  font-size: 11px;
  padding: 5px 30px 5px 8px;
  background: rgba(0, 0, 0, 0.4);
}

.char-count {
  position: absolute;
  right: 8px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.desc-textarea {
  font-size: 11px;
  padding: 5px 8px;
  background: rgba(0, 0, 0, 0.4);
  resize: none;
}

.actions-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.btn-full {
  width: 100%;
  justify-content: center;
  padding: 7px;
}

.stage-commit-btn {
  background: #2ed573;
  color: #0d1117;
  font-weight: 700;
}

.status-alert {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  padding: 5px 8px;
  border-radius: 3px;
}

.alert-danger {
  background: rgba(255, 71, 87, 0.15);
  color: var(--danger);
}

.alert-success {
  background: rgba(46, 213, 115, 0.15);
  color: var(--success);
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

.diff-modal {
  width: 680px;
  height: 480px;
  display: flex;
  flex-direction: column;
  background: var(--bg-panel);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 12px;
}

.modal-header .title {
  font-family: var(--font-mono);
  flex: 1;
}

.modal-body {
  flex: 1;
  overflow: auto;
  padding: 12px;
  background: rgba(0, 0, 0, 0.4);
}

.text-success { color: var(--success); }
.text-danger { color: var(--danger); }
.text-warning { color: var(--warning); }
.btn-xs { padding: 3px 6px; font-size: 10px; }
.spinning { animation: spin 1s linear infinite; }

@keyframes spin { 100% { transform: rotate(360deg); } }
</style>
