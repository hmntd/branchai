<script setup lang="ts">
import { ref, computed, reactive, watch } from 'vue';
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
  GitCommit,
} from '@lucide/vue';
import FileContextMenu from './FileContextMenu.vue';
import TreeNodes from './TreeNodes.vue';
import { GraphNode } from './BranchGraph.vue';
import BaseBadge from '../atoms/BaseBadge.vue';
import BaseButton from '../atoms/BaseButton.vue';

export interface FileStatus {
  path: string;
  status: string;
  staged: boolean;
}

const props = defineProps<{
  repoPath: string;
  files: FileStatus[];
  currentBranch: string;
  selectedCommit?: GraphNode | null;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
  (e: 'selectWip'): void;
}>();

const selectedFile = ref<FileStatus | null>(null);
const diffText = ref<string>('');
const loadingDiff = ref<boolean>(false);
const showDiffModal = ref<boolean>(false);

// Commit files state
const commitFiles = ref<FileStatus[]>([]);
const loadingCommitFiles = ref<boolean>(false);

async function loadCommitFiles() {
  if (!props.selectedCommit) {
    commitFiles.value = [];
    return;
  }
  loadingCommitFiles.value = true;
  try {
    const res: FileStatus[] = await invoke('get_commit_files', {
      repoPath: props.repoPath,
      commitId: props.selectedCommit.id,
    });
    commitFiles.value = res;
  } catch (err) {
    console.error('Failed to load commit files:', err);
    commitFiles.value = [];
  } finally {
    loadingCommitFiles.value = false;
  }
}

watch(
  () => props.selectedCommit?.id,
  () => {
    loadCommitFiles();
  },
  { immediate: true }
);

// Context menu state
const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxFile = ref<FileStatus | null>(null);

function openContextMenu(e: MouseEvent, file: FileStatus) {
  e.preventDefault();
  ctxFile.value = file;
  ctxX.value = e.clientX;
  ctxY.value = e.clientY;
  ctxVisible.value = true;
}

function closeContextMenu() {
  ctxVisible.value = false;
}

// View mode (path | tree)
const viewMode = ref<'path' | 'tree'>('path');

// Tree builder
interface TreeNode {
  name: string;
  fullPath: string;
  isDir: boolean;
  children: TreeNode[];
  file?: FileStatus;
}

// Tracks open/closed state of folders. Defaults to open.
const collapsedFolders = reactive<Record<string, boolean>>({});

function toggleFolder(fullPath: string) {
  collapsedFolders[fullPath] = !collapsedFolders[fullPath];
}

function buildTree(files: FileStatus[]): TreeNode[] {
  const root: TreeNode = { name: '', fullPath: '', isDir: true, children: [] };

  for (const file of files) {
    const parts = file.path.split('/');
    let node = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const isLast = i === parts.length - 1;
      const fullPath = parts.slice(0, i + 1).join('/');
      let child = node.children.find((c) => c.name === part);
      if (!child) {
        child = { name: part, fullPath, isDir: !isLast, children: [] };
        node.children.push(child);
      }
      if (isLast) {
        child.isDir = false;
        child.file = file;
      }
      node = child;
    }
  }

  // Sort: folders first, then files
  function sortChildren(n: TreeNode) {
    n.children.sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    n.children.forEach(sortChildren);
  }
  sortChildren(root);
  return root.children;
}

const unstagedTree = computed(() => buildTree(unstagedFiles.value));
const stagedTree = computed(() => buildTree(stagedFiles.value));
const commitTree = computed(() => buildTree(commitFiles.value));

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
    let diff: string;
    if (props.selectedCommit) {
      diff = await invoke('get_commit_file_diff', {
        repoPath: props.repoPath,
        commitId: props.selectedCommit.id,
        filePath: file.path,
      });
    } else {
      diff = await invoke('get_file_diff', {
        repoPath: props.repoPath,
        filePath: file.path,
        staged: file.staged,
      });
    }
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

// Diff parser: produces structured rows with line numbers
interface DiffRow {
  type: 'add' | 'remove' | 'context' | 'hunk' | 'meta';
  oldNo: number | null;
  newNo: number | null;
  content: string;
}

const parsedDiffLines = computed((): DiffRow[] => {
  const rows: DiffRow[] = [];
  let oldLine = 0;
  let newLine = 0;

  for (const raw of diffText.value.split('\n')) {
    // Hunk header: @@ -a,b +c,d @@
    const hunkMatch = raw.match(/^@@[^+\-]*-(\d+)(?:,\d+)?\s+\+(\d+)(?:,\d+)?/);
    if (hunkMatch) {
      oldLine = parseInt(hunkMatch[1], 10);
      newLine = parseInt(hunkMatch[2], 10);
      rows.push({ type: 'hunk', oldNo: null, newNo: null, content: raw });
      continue;
    }

    if (raw.startsWith('+')) {
      rows.push({ type: 'add', oldNo: null, newNo: newLine++, content: raw.slice(1) });
    } else if (raw.startsWith('-')) {
      rows.push({ type: 'remove', oldNo: oldLine++, newNo: null, content: raw.slice(1) });
    } else if (raw === '' || raw === '\\ No newline at end of file') {
      rows.push({ type: 'meta', oldNo: null, newNo: null, content: raw });
    } else {
      // context line (starts with space or is plain for untracked files)
      const text = raw.startsWith(' ') ? raw.slice(1) : raw;
      rows.push({ type: 'context', oldNo: oldLine++, newNo: newLine++, content: text });
    }
  }
  return rows;
});
</script>

<template>
  <aside class="right-staging-panel gk-panel">
    <!-- Header: Commit view or WIP view -->
    <div v-if="selectedCommit" class="panel-header">
      <span class="changes-title">
        {{ commitFiles.length }} file changes in
        <BaseBadge variant="branch">{{ selectedCommit.short_id }}</BaseBadge>
      </span>

      <div class="view-toggles">
        <BaseButton size="xs" variant="secondary" :active="viewMode === 'path'" @click="viewMode = 'path'">
          <ListFilter :size="11" /> Path
        </BaseButton>
        <BaseButton size="xs" variant="secondary" :active="viewMode === 'tree'" @click="viewMode = 'tree'">
          <FolderTree :size="11" /> Tree
        </BaseButton>
      </div>
    </div>
    <div v-else class="panel-header">
      <span class="changes-title">
        {{ files.length }} file changes on
        <BaseBadge variant="branch">{{ currentBranch || 'master' }}</BaseBadge>
      </span>

      <div class="view-toggles">
        <BaseButton size="xs" variant="secondary" :active="viewMode === 'path'" @click="viewMode = 'path'">
          <ListFilter :size="11" /> Path
        </BaseButton>
        <BaseButton size="xs" variant="secondary" :active="viewMode === 'tree'" @click="viewMode = 'tree'">
          <FolderTree :size="11" /> Tree
        </BaseButton>
      </div>
    </div>

    <!-- Files Section for Commit View -->
    <div v-if="selectedCommit" class="files-container">
      <div class="accordion-section">
        <div class="acc-header">
          <span class="acc-title">Changed Files ({{ commitFiles.length }})</span>
        </div>

        <!-- PATH view -->
        <div v-if="viewMode === 'path'" class="file-list">
          <div v-for="file in commitFiles" :key="file.path" class="file-row" @click="inspectFileDiff(file)"
            @contextmenu.prevent="openContextMenu($event, file)">
            <span class="status-prefix" :class="getFileIconClass(file.status)">{{ getFilePrefix(file.status) }}</span>
            <span class="file-path">{{ file.path }}</span>
          </div>
          <div v-if="commitFiles.length === 0 && !loadingCommitFiles" class="empty-list">No files changed in this commit
          </div>
          <div v-if="loadingCommitFiles" class="empty-list">Loading commit files...</div>
        </div>

        <!-- TREE view -->
        <div v-if="viewMode === 'tree'" class="file-list">
          <TreeNodes :nodes="commitTree" :staged="false" :read-only="true" :collapsed-folders="collapsedFolders"
            @toggle-folder="toggleFolder" @inspect="inspectFileDiff" @context-menu="openContextMenu"
            :get-icon-class="getFileIconClass" :get-prefix="getFilePrefix" />
          <div v-if="commitFiles.length === 0 && !loadingCommitFiles" class="empty-list">No files changed in this commit
          </div>
          <div v-if="loadingCommitFiles" class="empty-list">Loading commit files...</div>
        </div>
      </div>
    </div>

    <!-- Files Section for WIP View -->
    <div v-else class="files-container">
      <!-- Unstaged Files Accordion -->
      <div class="accordion-section">
        <div class="acc-header" @click="isUnstagedOpen = !isUnstagedOpen">
          <component :is="isUnstagedOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="acc-title">Unstaged Files ({{ unstagedFiles.length }})</span>
          <BaseButton v-if="unstagedFiles.length > 0" variant="primary" size="xs" class="stage-all-btn"
            @click.stop="stageAll">
            Stage All Changes
          </BaseButton>
        </div>

        <!-- PATH view -->
        <div v-if="isUnstagedOpen && viewMode === 'path'" class="file-list">
          <div v-for="file in unstagedFiles" :key="file.path" class="file-row" @click="inspectFileDiff(file)"
            @contextmenu.prevent="openContextMenu($event, file)">
            <span class="status-prefix" :class="getFileIconClass(file.status)">{{ getFilePrefix(file.status) }}</span>
            <span class="file-path">{{ file.path }}</span>
            <button class="stage-btn" @click.stop="stageFile(file)" title="Stage File">
              <Plus :size="11" />
            </button>
          </div>
          <div v-if="unstagedFiles.length === 0" class="empty-list">No unstaged changes</div>
        </div>

        <!-- TREE view -->
        <div v-if="isUnstagedOpen && viewMode === 'tree'" class="file-list">
          <TreeNodes :nodes="unstagedTree" :staged="false" :collapsed-folders="collapsedFolders"
            @toggle-folder="toggleFolder" @inspect="inspectFileDiff" @stage="stageFile" @context-menu="openContextMenu"
            :get-icon-class="getFileIconClass" :get-prefix="getFilePrefix" />
          <div v-if="unstagedFiles.length === 0" class="empty-list">No unstaged changes</div>
        </div>
      </div>

      <!-- Staged Files Accordion -->
      <div class="accordion-section">
        <div class="acc-header" @click="isStagedOpen = !isStagedOpen">
          <component :is="isStagedOpen ? ChevronDown : ChevronRight" :size="11" />
          <span class="acc-title">Staged Files ({{ stagedFiles.length }})</span>
        </div>

        <!-- PATH view -->
        <div v-if="isStagedOpen && viewMode === 'path'" class="file-list">
          <div v-for="file in stagedFiles" :key="file.path" class="file-row staged" @click="inspectFileDiff(file)"
            @contextmenu.prevent="openContextMenu($event, file)">
            <span class="status-prefix text-success">✓</span>
            <span class="file-path">{{ file.path }}</span>
            <button class="stage-btn" @click.stop="unstageFile(file)" title="Unstage File">
              <Minus :size="11" />
            </button>
          </div>
          <div v-if="stagedFiles.length === 0" class="empty-list">No staged changes</div>
        </div>

        <!-- TREE view -->
        <div v-if="isStagedOpen && viewMode === 'tree'" class="file-list">
          <TreeNodes :nodes="stagedTree" :staged="true" :collapsed-folders="collapsedFolders"
            @toggle-folder="toggleFolder" @inspect="inspectFileDiff" @unstage="unstageFile"
            @context-menu="openContextMenu" :get-icon-class="getFileIconClass" :get-prefix="getFilePrefix" />
          <div v-if="stagedFiles.length === 0" class="empty-list">No staged changes</div>
        </div>
      </div>
    </div>

    <!-- Commit Info Box (for Commit mode) -->
    <div v-if="selectedCommit" class="commit-box gk-panel">
      <div class="commit-box-header">
        <span class="commit-icon-label">
          <GitCommit :size="13" /> Commit {{ selectedCommit.short_id }}
        </span>
      </div>
      <div class="commit-details-body">
        <div class="commit-msg-preview">{{ selectedCommit.message }}</div>
        <div class="commit-author-tag">By <strong>{{ selectedCommit.author }}</strong></div>
      </div>
      <BaseButton variant="secondary" class="btn-full" @click="emit('selectWip')">
        Show WIP / Untracked Changes
      </BaseButton>
    </div>

    <!-- GitKraken Commit Composer Box (for WIP mode) -->
    <div v-else class="commit-box gk-panel">
      <div class="commit-box-header">
        <span class="commit-icon-label">-o- Commit</span>
      </div>

      <label class="checkbox-label">
        <input type="checkbox" v-model="amendCommit" />
        <span>Amend previous commit</span>
      </label>

      <div class="inputs-group">
        <div class="summary-wrap">
          <input v-model="commitSummary" placeholder="Commit summary" class="summary-input" />
          <span class="char-count" :class="{ 'text-danger': summaryCharCount < 0 }">
            {{ summaryCharCount }}
          </span>
        </div>
        <textarea v-model="commitDescription" placeholder="Description" rows="3" class="desc-textarea"></textarea>
      </div>

      <div v-if="statusMsg" class="status-alert" :class="statusError ? 'alert-danger' : 'alert-success'">
        <AlertCircle v-if="statusError" :size="12" />
        <Check v-else :size="12" />
        <span>{{ statusMsg }}</span>
      </div>

      <div class="actions-group">
        <!-- Purple "Compose commits with AI" button -->
        <BaseButton variant="ai" class="btn-full" @click="composeAiCommit" :disabled="isGeneratingAi">
          <RefreshCw v-if="isGeneratingAi" :size="13" class="spinning" />
          <Sparkles v-else :size="13" />
          + Compose commits with AI
        </BaseButton>

        <!-- Green Stage & Commit Button -->
        <BaseButton variant="primary" class="btn-full stage-commit-btn" @click="executeCommit"
          :disabled="isCommitting || !commitSummary.trim() || stagedFiles.length === 0">
          <CheckCircle2 :size="13" />
          {{ isCommitting ? 'Committing...' : `-o- Stage Changes to Commit (${stagedFiles.length})` }}
        </BaseButton>
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
            <div v-for="(row, idx) in parsedDiffLines" :key="idx" class="diff-line" :class="{
              'diff-add': row.type === 'add',
              'diff-remove': row.type === 'remove',
              'diff-hunk': row.type === 'hunk',
              'diff-meta': row.type === 'meta',
            }">
              <span class="diff-gutter diff-gutter-old">{{ row.oldNo ?? '' }}</span>
              <span class="diff-gutter diff-gutter-new">{{ row.newNo ?? '' }}</span>
              <span class="diff-sign">{{
                row.type === 'add' ? '+' :
                  row.type === 'remove' ? '-' :
                    row.type === 'hunk' ? '' : ' '
              }}</span>
              <span class="diff-content">{{ row.content }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </aside>

  <!-- File Context Menu -->
  <FileContextMenu :visible="ctxVisible" :x="ctxX" :y="ctxY" :file="ctxFile" :repo-path="repoPath"
    @close="closeContextMenu" @refresh="emit('refresh')" />
</template>

<style scoped src="../../styles/organisms/RightStagingPanel.css"></style>
