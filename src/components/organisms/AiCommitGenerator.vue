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
      <button @click="generateCommitMsg" class="btn btn-ai btn-sm" :disabled="isGenerating || stagedFilesCount === 0">
        <RefreshCw v-if="isGenerating" :size="14" class="spinning" />
        <Sparkles v-else :size="14" />
        {{ isGenerating ? 'Analyzing Diff...' : 'Generate with AI' }}
      </button>
    </div>

    <div class="card-body">
      <textarea v-model="commitMessage" placeholder="Enter commit message or click 'Generate with AI'..." rows="4"
        class="commit-textarea"></textarea>

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
      <button @click="handleCommit" class="btn btn-primary btn-sm"
        :disabled="isCommitting || !commitMessage.trim() || stagedFilesCount === 0">
        <Send :size="14" />
        {{ isCommitting ? 'Committing...' : 'Commit Staged Changes' }}
      </button>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/AiCommitGenerator.css"></style>
