<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Sparkles, Send, Check, AlertCircle, RefreshCw } from '@lucide/vue';

const props = defineProps<{
  repoPath: string;
  stagedFilesCount: number;
}>();

const emit = defineEmits<{
  (e: 'committed'): void;
}>();

const commitMessage = ref('');
const isGenerating = ref(false);
const isCommitting = ref(false);
const errorMessage = ref('');
const successMessage = ref('');

async function generateCommitMsg() {
  if (props.stagedFilesCount === 0) {
    errorMessage.value = 'Please stage at least one file before generating AI commit message.';
    return;
  }

  isGenerating.value = true;
  errorMessage.value = '';
  successMessage.value = '';

  try {
    const stagedDiff: string = await invoke('get_all_staged_diff', { repoPath: props.repoPath });
    if (!stagedDiff.trim()) {
      errorMessage.value = 'Staged diff is empty.';
      isGenerating.value = false;
      return;
    }

    const aiMsg: string = await invoke('generate_commit_message', { diff: stagedDiff });
    commitMessage.value = aiMsg.trim();
    successMessage.value = 'Commit message generated successfully!';
  } catch (err: any) {
    errorMessage.value = typeof err === 'string' ? err : err.message || 'Failed to generate commit message';
  } finally {
    isGenerating.value = false;
  }
}

async function handleCommit() {
  if (!commitMessage.value.trim()) {
    errorMessage.value = 'Commit message cannot be empty.';
    return;
  }

  isCommitting.value = true;
  errorMessage.value = '';

  try {
    await invoke('create_commit', { repoPath: props.repoPath, message: commitMessage.value.trim() });
    commitMessage.value = '';
    successMessage.value = 'Changes committed successfully!';
    emit('committed');
  } catch (err: any) {
    errorMessage.value = typeof err === 'string' ? err : err.message || 'Failed to commit';
  } finally {
    isCommitting.value = false;
  }
}
</script>

<template>
  <div class="ai-commit-card glass-panel">
    <div class="card-header">
      <div class="title-wrap">
        <Sparkles :size="16" class="sparkle-icon" />
        <span class="title">AI Commit Assistant</span>
      </div>
      <button
        @click="generateCommitMsg"
        class="btn btn-ai btn-sm"
        :disabled="isGenerating || stagedFilesCount === 0"
      >
        <RefreshCw v-if="isGenerating" :size="14" class="spinning" />
        <Sparkles v-else :size="14" />
        {{ isGenerating ? 'Analyzing Diff...' : 'Generate with AI' }}
      </button>
    </div>

    <div class="card-body">
      <textarea
        v-model="commitMessage"
        placeholder="Enter commit message or click 'Generate with AI'..."
        rows="4"
        class="commit-textarea"
      ></textarea>

      <div v-if="errorMessage" class="alert alert-danger">
        <AlertCircle :size="14" />
        <span>{{ errorMessage }}</span>
      </div>

      <div v-if="successMessage" class="alert alert-success">
        <Check :size="14" />
        <span>{{ successMessage }}</span>
      </div>
    </div>

    <div class="card-footer">
      <span class="staged-info">{{ stagedFilesCount }} file(s) staged</span>
      <button
        @click="handleCommit"
        class="btn btn-primary btn-sm"
        :disabled="isCommitting || !commitMessage.trim() || stagedFilesCount === 0"
      >
        <Send :size="14" />
        {{ isCommitting ? 'Committing...' : 'Commit Staged Changes' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.ai-commit-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: rgba(19, 24, 37, 0.95);
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 14px;
}

.sparkle-icon {
  color: #c084fc;
}

.commit-textarea {
  width: 100%;
  resize: vertical;
  font-family: var(--font-sans);
  font-size: 13px;
  line-height: 1.5;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.staged-info {
  font-size: 12px;
  color: var(--text-muted);
}

.alert {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  margin-top: 8px;
}

.alert-danger {
  background: rgba(248, 113, 113, 0.15);
  color: var(--danger);
  border: 1px solid rgba(248, 113, 113, 0.3);
}

.alert-success {
  background: rgba(52, 211, 153, 0.15);
  color: var(--success);
  border: 1px solid rgba(52, 211, 153, 0.3);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}
</style>
