<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ShieldCheck, Bug, Zap, Sparkles, RefreshCw, AlertTriangle, FileCode, X } from '@lucide/vue';
import BaseButton from '../atoms/BaseButton.vue';

const props = defineProps<{
  repoPath: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const isAnalyzing = ref(false);
const reviewResult = ref('');
const errorMessage = ref('');

async function runCodeReview() {
  isAnalyzing.value = true;
  errorMessage.value = '';
  reviewResult.value = '';

  try {
    const stagedDiff: string = await invoke('get_all_staged_diff', { repoPath: props.repoPath });
    let targetDiff = stagedDiff;

    if (!targetDiff.trim()) {
      targetDiff = 'No staged diff found. Please stage changes or select files to review.';
    }

    const reviewText: string = await invoke('analyze_code_review', { diff: targetDiff });
    reviewResult.value = reviewText;
  } catch (err: any) {
    errorMessage.value = typeof err === 'string' ? err : err.message || 'Failed to analyze code review.';
  } finally {
    isAnalyzing.value = false;
  }
}
</script>

<template>
  <div class="review-container">
    <div class="review-sidebar glass-panel">
      <div class="sidebar-header">
        <ShieldCheck :size="20" class="icon-accent" />
        <h3 class="header-title">AI Code Reviewer</h3>
        <button class="icon-btn close-btn" @click="emit('close')" title="Close & Back to Graph">
          <X :size="16" />
        </button>
      </div>
      <p class="sidebar-desc">
        Automatically inspect your Git diffs before pushing. The AI analyzes potential bugs, security flaws, performance
        optimizations, and code style.
      </p>

      <div class="categories">
        <div class="category-item">
          <Bug :size="16" class="icon-danger" />
          <span>Bug Detection & Edge Cases</span>
        </div>
        <div class="category-item">
          <ShieldCheck :size="16" class="icon-warning" />
          <span>Security & Vulnerabilities</span>
        </div>
        <div class="category-item">
          <Zap :size="16" class="icon-primary" />
          <span>Performance Optimization</span>
        </div>
        <div class="category-item">
          <FileCode :size="16" class="icon-success" />
          <span>Code Style & Best Practices</span>
        </div>
      </div>

      <BaseButton variant="ai" class="btn-full" @click="runCodeReview" :disabled="isAnalyzing">
        <RefreshCw v-if="isAnalyzing" :size="16" class="spinning" />
        <Sparkles v-else :size="16" />
        {{ isAnalyzing ? 'Running Analysis...' : 'Run Automated AI Review' }}
      </BaseButton>
    </div>

    <div class="review-main glass-panel">
      <div class="main-header">
        <Sparkles :size="18" class="icon-purple" />
        <span class="title">Review Findings & Suggestions</span>
        <BaseButton variant="secondary" size="xs" @click="emit('close')">
          <X :size="14" /> Back to Graph
        </BaseButton>
      </div>

      <div class="main-body">
        <div v-if="isAnalyzing" class="loading-box">
          <RefreshCw :size="32" class="spinning icon-purple" />
          <p>AI is scanning diffs and analyzing code structure...</p>
        </div>

        <div v-else-if="errorMessage" class="error-box">
          <AlertTriangle :size="24" class="icon-danger" />
          <p>{{ errorMessage }}</p>
        </div>

        <div v-else-if="reviewResult" class="markdown-output">
          <pre class="formatted-output">{{ reviewResult }}</pre>
        </div>

        <div v-else class="empty-state">
          <ShieldCheck :size="48" class="icon-dim" />
          <p>Click <strong>"Run Automated AI Review"</strong> to trigger an AI analysis on your current staged changes.
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped src="../../styles/organisms/AiCodeReview.css"></style>
