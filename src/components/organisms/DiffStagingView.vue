<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Plus, Minus, Layers, FileText, CheckCircle2 } from '@lucide/vue';
import AiCommitGenerator from './AiCommitGenerator.vue';
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
          <div v-for="file in stagedFiles" :key="file.path" class="file-item"
            :class="{ active: selectedFile?.path === file.path && selectedFile?.staged }" @click="loadDiff(file)">
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
          <BaseButton v-if="unstagedFiles.length > 0" variant="secondary" size="xs" @click="stageAll">
            Stage All
          </BaseButton>
        </div>
        <div class="file-list">
          <div v-for="file in unstagedFiles" :key="file.path" class="file-item"
            :class="{ active: selectedFile?.path === file.path && !selectedFile?.staged }" @click="loadDiff(file)">
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
      <AiCommitGenerator :repo-path="repoPath" :staged-files-count="stagedFiles.length" @committed="emit('refresh')" />
    </div>

    <!-- Right Main: File Diff Viewer -->
    <div class="diff-panel glass-panel">
      <div class="diff-header" v-if="selectedFile">
        <FileText :size="16" class="icon-primary" />
        <span class="diff-title">{{ selectedFile.path }}</span>
        <BaseBadge :variant="selectedFile.staged ? 'staged' : 'modified'">
          {{ selectedFile.staged ? 'Staged' : 'Unstaged' }}
        </BaseBadge>
      </div>

      <div class="diff-body" v-if="selectedFile">
        <div v-if="loadingDiff" class="loading-state">Loading diff...</div>
        <div v-else-if="diffText" class="diff-content">
          <div v-for="(line, idx) in diffText.split('\n')" :key="idx" class="diff-line" :class="{
            'diff-add': line.startsWith('+'),
            'diff-remove': line.startsWith('-'),
            'diff-header': line.startsWith('@'),
          }">
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

<style scoped src="../../styles/organisms/DiffStagingView.css"></style>
