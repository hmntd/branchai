<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Plus, Minus, Layers, FileText, CheckCircle2 } from '@lucide/vue';
import AiCommitGenerator from './AiCommitGenerator.vue';

export interface FileStatus {
  path: string;
  status: string;
  staged: boolean;
}

const props = defineProps<{
  repoPath: string;
  files: FileStatus[];
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
}>();

const selectedFile = ref<FileStatus | null>(null);
const diffText = ref<string>('');
const loadingDiff = ref<boolean>(false);

const stagedFiles = computed(() => props.files.filter((f) => f.staged));
const unstagedFiles = computed(() => props.files.filter((f) => !f.staged));

async function loadDiff(file: FileStatus) {
  selectedFile.value = file;
  loadingDiff.value = true;
  try {
    const diff: string = await invoke('get_file_diff', {
      repoPath: props.repoPath,
      filePath: file.path,
      staged: file.staged,
    });
    diffText.value = diff;
  } catch (e) {
    diffText.value = 'Failed to load diff.';
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

watch(
  () => props.files,
  (newFiles) => {
    if (newFiles.length > 0 && !selectedFile.value) {
      loadDiff(newFiles[0]);
    }
  },
  { immediate: true }
);
</script>

<template>
  <div class="staging-container">
    <!-- Left Sidebar: Staged & Unstaged File Lists -->
    <div class="file-panel glass-panel">
      <!-- Staged Files -->
      <div class="panel-section">
        <div class="section-header">
          <span class="section-title">
            <CheckCircle2 :size="14" class="icon-success" />
            Staged Changes ({{ stagedFiles.length }})
          </span>
        </div>
        <div class="file-list">
          <div
            v-for="file in stagedFiles"
            :key="file.path"
            class="file-item"
            :class="{ active: selectedFile?.path === file.path && selectedFile?.staged }"
            @click="loadDiff(file)"
          >
            <FileText :size="14" class="file-icon" />
            <span class="file-name">{{ file.path }}</span>
            <button class="action-btn" @click.stop="unstageFile(file)" title="Unstage File">
              <Minus :size="12" />
            </button>
          </div>
          <div v-if="stagedFiles.length === 0" class="empty-text">No staged files</div>
        </div>
      </div>

      <!-- Unstaged Files -->
      <div class="panel-section">
        <div class="section-header">
          <span class="section-title">
            <Layers :size="14" class="icon-warning" />
            Unstaged Changes ({{ unstagedFiles.length }})
          </span>
          <button v-if="unstagedFiles.length > 0" class="btn btn-secondary btn-xs" @click="stageAll">
            Stage All
          </button>
        </div>
        <div class="file-list">
          <div
            v-for="file in unstagedFiles"
            :key="file.path"
            class="file-item"
            :class="{ active: selectedFile?.path === file.path && !selectedFile?.staged }"
            @click="loadDiff(file)"
          >
            <FileText :size="14" class="file-icon" />
            <span class="file-name">{{ file.path }}</span>
            <button class="action-btn" @click.stop="stageFile(file)" title="Stage File">
              <Plus :size="12" />
            </button>
          </div>
          <div v-if="unstagedFiles.length === 0" class="empty-text">No unstaged changes</div>
        </div>
      </div>

      <!-- AI Commit Widget -->
      <AiCommitGenerator
        :repo-path="repoPath"
        :staged-files-count="stagedFiles.length"
        @committed="emit('refresh')"
      />
    </div>

    <!-- Right Main: File Diff Viewer -->
    <div class="diff-panel glass-panel">
      <div class="diff-header" v-if="selectedFile">
        <FileText :size="16" class="icon-primary" />
        <span class="diff-title">{{ selectedFile.path }}</span>
        <span class="badge" :class="selectedFile.staged ? 'badge-staged' : 'badge-modified'">
          {{ selectedFile.staged ? 'Staged' : 'Unstaged' }}
        </span>
      </div>

      <div class="diff-body" v-if="selectedFile">
        <div v-if="loadingDiff" class="loading-state">Loading diff...</div>
        <div v-else-if="diffText" class="diff-content">
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
        <div v-else class="empty-diff">Select a file to view code diff</div>
      </div>

      <div class="empty-panel" v-else>
        <FileText :size="48" class="icon-dim" />
        <p>No changes selected. Modify files in your repository to stage and commit.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.staging-container {
  display: grid;
  grid-template-columns: 380px 1fr;
  gap: 16px;
  height: 100%;
  padding: 16px;
}

.file-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  overflow-y: auto;
}

.panel-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  font-weight: 600;

}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 200px;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  transition: background 0.15s;
}

.file-item:hover {
  background: var(--bg-card-hover);
}

.file-item.active {
  background: rgba(56, 189, 248, 0.12);
  color: var(--primary);
}

.file-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-mono);
}

.action-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
}

.action-btn:hover {
  color: var(--text-main);
  background: rgba(255, 255, 255, 0.1);
}

.empty-text {
  font-size: 12px;
  color: var(--text-dim);
  font-style: italic;
  padding: 4px 8px;
}

.diff-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.diff-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.diff-title {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 500;
  flex: 1;
}

.diff-body {
  flex: 1;
  overflow: auto;
  padding: 16px;
  background: rgba(0, 0, 0, 0.3);
}

.diff-content {
  display: flex;
  flex-direction: column;
}

.empty-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--text-dim);
  text-align: center;
  padding: 20px;
}

.btn-xs {
  padding: 2px 8px;
  font-size: 11px;
}
</style>
